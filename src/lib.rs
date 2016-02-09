extern crate libc;
use std::ffi::CString;
use std::mem::size_of;

#[allow(dead_code,
        non_snake_case,
        non_camel_case_types,
        non_upper_case_globals)]
pub mod libpq;


const CTRUE: libpq::_bool = 1;
const CFALSE: libpq::_bool = 0;

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
        let last_relid = libpq::palloc0(size_of::<libpq::Oid>() as u64);
        (*ctx).output_plugin_private = last_relid;
        (*options).output_type = libpq::OUTPUT_PLUGIN_TEXTUAL_OUTPUT;
    }
}

#[allow(unused_variables)]
extern fn begin(ctx: *mut libpq::Struct_LogicalDecodingContext,
                txn: *mut libpq::ReorderBufferTXN) {
    unsafe {
        let s = CString::new("{ \"begin\": %u }").unwrap();
        libpq::OutputPluginPrepareWrite(ctx, CTRUE);
        libpq::appendStringInfo((*ctx).out, s.as_ptr(), (*txn).xid);
        libpq::OutputPluginWrite(ctx, CTRUE);
    }
}

#[allow(unused_variables)]
extern fn change(ctx: *mut libpq::Struct_LogicalDecodingContext,
                 txn: *mut libpq::ReorderBufferTXN,
                 relation: libpq::Relation,
                 change: *mut libpq::ReorderBufferChange) {
    unsafe {
        let relid = (*relation).rd_id;
        let last_relid: *mut libpq::Oid =
            (*ctx).output_plugin_private as *mut libpq::Oid;
        if *last_relid != relid {
            libpq::OutputPluginPrepareWrite(ctx, CFALSE);
            append_schema(relation, (*ctx).out);
            libpq::OutputPluginWrite(ctx, CFALSE);
            *last_relid = relid;
        }
        libpq::OutputPluginPrepareWrite(ctx, CTRUE);
        append_change(relation, change, (*ctx).out);
        libpq::OutputPluginWrite(ctx, CTRUE);
    }
}

#[allow(unused_variables)]
extern fn commit(ctx: *mut libpq::Struct_LogicalDecodingContext,
                 txn: *mut libpq::ReorderBufferTXN,
                 lsn: libpq::XLogRecPtr) {
    unsafe {
        let s = CString::new("{ \"commit\": %u, \"t\": \"%s\" }").unwrap();
        let t = libpq::timestamptz_to_str((*txn).commit_time);
        libpq::OutputPluginPrepareWrite(ctx, CTRUE);
        libpq::appendStringInfo((*ctx).out, s.as_ptr(), (*txn).xid, t);
        libpq::OutputPluginWrite(ctx, CTRUE);
    }
}

#[allow(unused_variables)]
extern fn shutdown(ctx: *mut libpq::Struct_LogicalDecodingContext) {
    unsafe {
        libpq::pfree((*ctx).output_plugin_private);
    }
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

unsafe fn append_schema(relation: libpq::Relation, out: libpq::StringInfo) {
    let relid = (*relation).rd_id;
    let tupdesc = (*relation).rd_att;
    let name = libpq::get_rel_name(relid);
    let ns = libpq::get_namespace_name(libpq::get_rel_namespace(relid));
    let qualified_name = libpq::quote_qualified_identifier(ns, name);
    append("{ \"table\": ", out);
    append("\"", out);
    libpq::appendStringInfoString(out, qualified_name);
    append("\"", out);
    append(",", out);
    append(" \"schema\": ", out);
    append("[", out);
    let fmt = CString::new("{\"%s\":\"%s\"}").unwrap();
    let mut first: bool = true;
    for i in 0..(*tupdesc).natts {
        let attr = *(*tupdesc).attrs.offset(i as isize);
        let num = (*attr).attnum;
        if (*attr).attisdropped == 1 || num <= 0 {
            continue;
        }
        let col = libpq::get_attname(relid, num);
        let typ = libpq::format_type_be(libpq::get_atttype(relid, num));
        if !first {
            append(",", out);
        } else {
            first = false;
        }
        libpq::appendStringInfo(out, fmt.as_ptr(), col, typ);
    }
    append("]", out);
    append(" }", out);
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
