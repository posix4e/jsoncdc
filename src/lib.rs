extern crate libc;
use std::ffi::CString;

#[allow(dead_code,
        non_snake_case,
        non_camel_case_types,
        non_upper_case_globals)]
pub mod libpq;


// Implementation of initialization and callbacks.

pub unsafe extern fn init(cb: *mut libpq::OutputPluginCallbacks) {
    (*cb).startup_cb = Some(startup);
    (*cb).begin_cb = Some(begin);
    (*cb).change_cb = Some(change);
    (*cb).commit_cb = Some(commit);
    (*cb).shutdown_cb = Some(shutdown);
}

#[allow(unused_variables)]
extern fn startup(ctx: *mut libpq::Struct_LogicalDecodingContext,
                  options: *mut libpq::OutputPluginOptions,
                  is_init: libpq::_bool) {
    unsafe {
        (*options).output_type = libpq::OUTPUT_PLUGIN_TEXTUAL_OUTPUT;
    }
}

#[allow(unused_variables)]
extern fn begin(ctx: *mut libpq::Struct_LogicalDecodingContext,
                txn: *mut libpq::ReorderBufferTXN) {
    unsafe {
        let is_last = 1;                                  // True in C language
        let s = CString::new("{ \"begin\": %u }").unwrap();
        libpq::OutputPluginPrepareWrite(ctx, is_last);
        libpq::appendStringInfo((*ctx).out, s.as_ptr(), (*txn).xid);
        libpq::OutputPluginWrite(ctx, is_last);
    }
}

#[allow(unused_variables)]
extern fn change(ctx: *mut libpq::Struct_LogicalDecodingContext,
                 txn: *mut libpq::ReorderBufferTXN,
                 relation: libpq::Relation,
                 change: *mut libpq::ReorderBufferChange) {
    unsafe {
        let last = 1;                                     // True in C language
        libpq::OutputPluginPrepareWrite(ctx, last);
        append_change(relation, change, (*ctx).out);
        libpq::OutputPluginWrite(ctx, last);
    }
}

#[allow(unused_variables)]
extern fn commit(ctx: *mut libpq::Struct_LogicalDecodingContext,
                 txn: *mut libpq::ReorderBufferTXN,
                 lsn: libpq::XLogRecPtr) {
    unsafe {
        let last = 1;                                     // True in C language
        let s = CString::new("{ \"commit\": %u, \"t\": \"%s\" }").unwrap();
        let t = libpq::timestamptz_to_str((*txn).commit_time);
        libpq::OutputPluginPrepareWrite(ctx, last);
        libpq::appendStringInfo((*ctx).out, s.as_ptr(), (*txn).xid, t);
        libpq::OutputPluginWrite(ctx, last);
    }
}

#[allow(unused_variables)]
extern fn shutdown(ctx: *mut libpq::Struct_LogicalDecodingContext) {
  // Do nothing.
}


unsafe fn append_change(relation: libpq::Relation,
                        change: *mut libpq::ReorderBufferChange,
                        out: libpq::StringInfo) {
    let tuple_desc = (*relation).rd_att;
    let tuples = (*change).data.tp();
    let tuple_new = (*tuples).newtuple;
    let tuple_old = (*tuples).oldtuple;
    let token = match (*change).action {
        libpq::REORDER_BUFFER_CHANGE_INSERT => "insert",
        libpq::REORDER_BUFFER_CHANGE_UPDATE => "update",
        libpq::REORDER_BUFFER_CHANGE_DELETE => "delete",
        _ => panic!("Unrecognized change action!")
    };
    append("{ ", out);
    append("\"", out);
    append(token, out);
    append("\": ", out);
    append_tuple_buf_as_json(tuple_new, tuple_desc, out);
    if !tuple_old.is_null() {
        append(", \"@\": ", out);
        append_tuple_buf_as_json(tuple_old, tuple_desc, out);
    }
    append(" }", out);
}

unsafe fn append_tuple_buf_as_json(data: *mut libpq::ReorderBufferTupleBuf,
                                   desc: libpq::TupleDesc,
                                   out: libpq::StringInfo) {
    if !data.is_null() {
        let heap_tuple = &mut (*data).tuple;
        let datum = libpq::heap_copy_tuple_as_datum(heap_tuple, desc);
        let empty_oid: libpq::Oid = 0;
        let json = libpq::DirectFunctionCall1Coll(Some(row_to_json),
                                                  empty_oid,
                                                  datum);
        let ptr = json as *const libpq::Struct_varlena;
        let text = libpq::text_to_cstring(ptr);
        libpq::appendStringInfoString(out, text);
    } else {
        append("{}", out);
    }
}

unsafe fn append<T: Into<Vec<u8>>>(t: T, out: libpq::StringInfo) {
    libpq::appendStringInfoString(out, CString::new(t).unwrap().as_ptr());
}

extern fn row_to_json(fcinfo: libpq::FunctionCallInfo) -> libpq::Datum {
    // We wrap the unsafe call to make it safe.
    unsafe {
        libpq::row_to_json(fcinfo)
    }
}


// Symbols Postgres needs to find.

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern fn _PG_init() { }

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern fn
    _PG_output_plugin_init(cb: *mut libpq::OutputPluginCallbacks) { init(cb); }
