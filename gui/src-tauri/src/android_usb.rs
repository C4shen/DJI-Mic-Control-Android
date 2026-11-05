//! JNI entry points called by `org.djimic.control.DjiUsbManager` on Android.

use device::DeviceError;
use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jboolean, jbyteArray, jint, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;

use crate::{global_manager, take_android_usb_permission_request};

fn jstr_to_string(env: &mut JNIEnv, s: &JString) -> Option<String> {
    if s.is_null() {
        return None;
    }
    env.get_string(s).ok().map(Into::into)
}

#[no_mangle]
pub extern "system" fn Java_org_djimic_control_DjiUsbManager_nativeOnProbe<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    present: jint,
    accessible: jint,
    permission_issue: jboolean,
) -> jboolean {
    let mgr = global_manager();
    mgr.bridge_set_probe(
        present.max(0) as usize,
        accessible.max(0) as usize,
        permission_issue == JNI_TRUE,
    );
    if take_android_usb_permission_request() {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_org_djimic_control_DjiUsbManager_nativeOnDeviceConnected<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    device_id: JString<'local>,
    vid: jint,
    pid: jint,
) -> jboolean {
    let Some(id) = jstr_to_string(&mut env, &device_id) else {
        return JNI_FALSE;
    };
    let mgr = global_manager();
    if mgr.bridge_connect(id, vid as u16, pid as u16) {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_org_djimic_control_DjiUsbManager_nativeOnDeviceDisconnected<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    device_id: JString<'local>,
) {
    let Some(id) = jstr_to_string(&mut env, &device_id) else {
        return;
    };
    let mgr = global_manager();
    mgr.bridge_disconnect(&id);
}

#[no_mangle]
pub extern "system" fn Java_org_djimic_control_DjiUsbManager_nativeFeedBytes<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    device_id: JString<'local>,
    data: JByteArray<'local>,
    len: jint,
) {
    let Some(id) = jstr_to_string(&mut env, &device_id) else {
        return;
    };
    let Ok(bytes) = env.convert_byte_array(&data) else {
        return;
    };
    let take_len = (len.max(0) as usize).min(bytes.len());
    if take_len > 0 {
        let mgr = global_manager();
        mgr.bridge_feed_bytes(&id, &bytes[..take_len]);
    }
}

#[no_mangle]
pub extern "system" fn Java_org_djimic_control_DjiUsbManager_nativeNextPacket<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    device_id: JString<'local>,
) -> jbyteArray {
    let Some(id) = jstr_to_string(&mut env, &device_id) else {
        return std::ptr::null_mut();
    };
    let mgr = global_manager();
    match mgr.bridge_next_packet(&id) {
        Some(pkt) => match env.byte_array_from_slice(&pkt) {
            Ok(arr) => arr.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "system" fn Java_org_djimic_control_DjiUsbManager_nativeCompletePacket<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    device_id: JString<'local>,
    ok: jboolean,
    err_msg: JString<'local>,
) {
    let Some(id) = jstr_to_string(&mut env, &device_id) else {
        return;
    };
    let res = if ok == JNI_TRUE {
        Ok(())
    } else {
        let msg = jstr_to_string(&mut env, &err_msg).unwrap_or_else(|| "bulkTransfer failed".to_string());
        Err(DeviceError::Usb(msg))
    };
    let mgr = global_manager();
    mgr.bridge_complete_packet(&id, res);
}
