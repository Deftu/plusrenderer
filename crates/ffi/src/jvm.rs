//! JNI for `org.polyfrost.plusrenderer.Renderer` (`java/`): the C ABI above, with Java strings in and
//! arrays copied out. The Java side checks handles and turns the -1 codes into exceptions.
#![allow(non_snake_case, clippy::missing_safety_doc, clippy::too_many_arguments)]

use jni::objects::{JClass, JFloatArray, JString};
use jni::sys::{jboolean, jbyteArray, jfloat, jfloatArray, jint, jintArray, jlong, jstring};
use jni::JNIEnv;

use super::*;

fn text(env: &mut JNIEnv, s: &JString) -> String {
    env.get_string(s).map(Into::into).unwrap_or_default()
}

fn len(s: &str) -> u32 {
    s.len() as u32
}

fn scene<'a>(r: jlong) -> &'a Scene {
    // SAFETY: the Java side only passes live handles from `nCreate`.
    unsafe { &(*(r as *const Renderer)).scene }
}

fn floats(env: &mut JNIEnv, data: &[f32]) -> jfloatArray {
    match env.new_float_array(data.len() as jint) {
        Ok(array) if env.set_float_array_region(&array, 0, data).is_ok() => array.into_raw(),
        _ => std::ptr::null_mut(),
    }
}

fn ints(env: &mut JNIEnv, data: &[i32]) -> jintArray {
    match env.new_int_array(data.len() as jint) {
        Ok(array) if env.set_int_array_region(&array, 0, data).is_ok() => array.into_raw(),
        _ => std::ptr::null_mut(),
    }
}

/// `DrawRun` / `MeshRun` are `repr(C)` structs of `u32`, read as their flat words.
fn words<T>(runs: &[T]) -> &[i32] {
    let n = std::mem::size_of_val(runs) / 4;
    // SAFETY: both run types are `repr(C)` and made only of `u32` fields.
    unsafe { std::slice::from_raw_parts(runs.as_ptr().cast(), n) }
}

#[no_mangle]
pub extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nCreate(
    _: JNIEnv,
    _: JClass,
) -> jlong {
    renderer_new() as jlong
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nFree(
    _: JNIEnv,
    _: JClass,
    r: jlong,
) {
    renderer_free(r as *mut Renderer);
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nError(
    env: JNIEnv,
    _: JClass,
    r: jlong,
) -> jstring {
    let error = &(*(r as *const Renderer)).error;
    env.new_string(error)
        .map_or(std::ptr::null_mut(), |s| s.into_raw())
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nLoadPlayer(
    _: JNIEnv,
    _: JClass,
    r: jlong,
    slot: jint,
    slim: jboolean,
) {
    renderer_load_player(r as *mut Renderer, slot as u32, slim as u32);
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nLoadCosmetic(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
    slot: jint,
    json: JString,
    body: JString,
    anchor: JString,
    image_width: jint,
    image_height: jint,
) -> jint {
    let (json, body, anchor) = (
        text(&mut env, &json),
        text(&mut env, &body),
        text(&mut env, &anchor),
    );
    renderer_load_cosmetic(
        r as *mut Renderer,
        slot as u32,
        json.as_ptr(),
        len(&json),
        body.as_ptr(),
        len(&body),
        anchor.as_ptr(),
        len(&anchor),
        image_width as u32,
        image_height as u32,
    )
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nLoadCape(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
    slot: jint,
    still_width: jint,
    still_height: jint,
    sheet_width: jint,
    sheet_height: jint,
    name: JString,
) -> jint {
    let name = text(&mut env, &name);
    renderer_load_cape(
        r as *mut Renderer,
        slot as u32,
        still_width as u32,
        still_height as u32,
        sheet_width as u32,
        sheet_height as u32,
        name.as_ptr(),
        len(&name),
    )
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nLoadElytra(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
    slot: jint,
    still_width: jint,
    still_height: jint,
    sheet_width: jint,
    sheet_height: jint,
    name: JString,
) -> jint {
    let name = text(&mut env, &name);
    renderer_load_elytra(
        r as *mut Renderer,
        slot as u32,
        still_width as u32,
        still_height as u32,
        sheet_width as u32,
        sheet_height as u32,
        name.as_ptr(),
        len(&name),
    )
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nBundlePart(
    mut env: JNIEnv,
    _: JClass,
    names: JString,
    kind: jint,
) -> jint {
    let names = text(&mut env, &names);
    bundle_part(names.as_ptr(), len(&names), kind as u32)
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nLoadAnimation(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
    slot: jint,
    json: JString,
) -> jint {
    let json = text(&mut env, &json);
    renderer_load_animation(r as *mut Renderer, slot as u32, json.as_ptr(), len(&json))
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nPlay(
    _: JNIEnv,
    _: JClass,
    r: jlong,
    slot: jint,
    at: jfloat,
) {
    renderer_play(r as *mut Renderer, slot as u32, at);
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nStop(
    _: JNIEnv,
    _: JClass,
    r: jlong,
    slot: jint,
) {
    renderer_stop(r as *mut Renderer, slot as u32);
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nSetElytraPose(
    _: JNIEnv,
    _: JClass,
    r: jlong,
    slot: jint,
    pose: jint,
    fall: jfloat,
) {
    renderer_set_elytra_pose(r as *mut Renderer, slot as u32, pose as u32, fall);
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nSetVisible(
    _: JNIEnv,
    _: JClass,
    r: jlong,
    slot: jint,
    visible: jboolean,
) {
    renderer_set_visible(r as *mut Renderer, slot as u32, visible as u32);
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nSetScale(
    _: JNIEnv,
    _: JClass,
    r: jlong,
    slot: jint,
    scale: jfloat,
) {
    renderer_set_scale(r as *mut Renderer, slot as u32, scale);
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nClear(
    _: JNIEnv,
    _: JClass,
    r: jlong,
    slot: jint,
) {
    renderer_clear(r as *mut Renderer, slot as u32);
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nSetLightLevel(
    _: JNIEnv,
    _: JClass,
    r: jlong,
    level: jfloat,
) {
    renderer_set_light_level(r as *mut Renderer, level);
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nPose(
    _: JNIEnv,
    _: JClass,
    r: jlong,
    seconds: jfloat,
) -> jint {
    renderer_pose(r as *mut Renderer, seconds)
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nSetGamePart(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
    name: JString,
    rest_x: jfloat,
    rest_y: jfloat,
    rest_z: jfloat,
    x: jfloat,
    y: jfloat,
    z: jfloat,
    x_rot: jfloat,
    y_rot: jfloat,
    z_rot: jfloat,
    x_scale: jfloat,
    y_scale: jfloat,
    z_scale: jfloat,
) {
    let name = text(&mut env, &name);
    renderer_set_game_part(
        r as *mut Renderer,
        name.as_ptr(),
        len(&name),
        rest_x,
        rest_y,
        rest_z,
        x,
        y,
        z,
        x_rot,
        y_rot,
        z_rot,
        x_scale,
        y_scale,
        z_scale,
    );
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nSetQuery(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
    name: JString,
    value: jfloat,
) {
    let name = text(&mut env, &name);
    renderer_set_query(r as *mut Renderer, name.as_ptr(), len(&name), value);
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nClearGameParts(
    _: JNIEnv,
    _: JClass,
    r: jlong,
) {
    renderer_clear_game_parts(r as *mut Renderer);
}

/// Nine floats (offset, rotation in radians, scale), or null if the slot has no such bone.
#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nPartDelta(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
    slot: jint,
    name: JString,
) -> jfloatArray {
    let name = text(&mut env, &name);
    let mut delta = [0.0f32; 9];

    if renderer_part_delta(
        r as *const Renderer,
        slot as u32,
        name.as_ptr(),
        len(&name),
        delta.as_mut_ptr(),
    ) < 0
    {
        return std::ptr::null_mut();
    }

    floats(&mut env, &delta)
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nBuildMesh(
    _: JNIEnv,
    _: JClass,
    r: jlong,
    space: jint,
) -> jint {
    renderer_build_mesh(r as *mut Renderer, space as u32)
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nBuild(
    _: JNIEnv,
    _: JClass,
    r: jlong,
    yaw: jfloat,
    pitch: jfloat,
    distance: jfloat,
    target_y: jfloat,
    width: jfloat,
    height: jfloat,
    seconds: jfloat,
) -> jint {
    renderer_build(
        r as *mut Renderer,
        yaw,
        pitch,
        distance,
        target_y,
        width,
        height,
        seconds,
    )
}

#[no_mangle]
pub extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nMeshPositions(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
) -> jfloatArray {
    floats(&mut env, &scene(r).mesh.positions)
}

#[no_mangle]
pub extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nMeshNormals(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
) -> jfloatArray {
    floats(&mut env, &scene(r).mesh.normals)
}

#[no_mangle]
pub extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nMeshUvs(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
) -> jfloatArray {
    floats(&mut env, &scene(r).mesh.uvs)
}

#[no_mangle]
pub extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nMeshGlow(
    env: JNIEnv,
    _: JClass,
    r: jlong,
) -> jbyteArray {
    env.byte_array_from_slice(&scene(r).mesh.glow)
        .map_or(std::ptr::null_mut(), |a| a.into_raw())
}

#[no_mangle]
pub extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nMeshRuns(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
) -> jintArray {
    ints(&mut env, words(&scene(r).mesh.runs))
}

#[no_mangle]
pub extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nPositions(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
) -> jfloatArray {
    floats(&mut env, &scene(r).buffers.positions)
}

#[no_mangle]
pub extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nUvs(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
) -> jfloatArray {
    floats(&mut env, &scene(r).buffers.uvs)
}

#[no_mangle]
pub extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nColors(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
) -> jintArray {
    ints(&mut env, words(&scene(r).buffers.colors))
}

#[no_mangle]
pub extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nIndices(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
) -> jintArray {
    let indices: Vec<i32> = scene(r).buffers.indices.iter().map(|&i| i as i32).collect();
    ints(&mut env, &indices)
}

#[no_mangle]
pub extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nRuns(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
) -> jintArray {
    ints(&mut env, words(&scene(r).buffers.runs))
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nPartId(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
    name: JString,
) -> jint {
    let name = text(&mut env, &name);
    renderer_part_id(r as *mut Renderer, name.as_ptr(), len(&name)) as jint
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nQueryId(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
    name: JString,
) -> jint {
    let name = text(&mut env, &name);
    renderer_query_id(r as *mut Renderer, name.as_ptr(), len(&name)) as jint
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nBuildSkin(
    _: JNIEnv,
    _: JClass,
    r: jlong,
) -> jint {
    renderer_build_skin(r as *mut Renderer)
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nLayout(
    _: JNIEnv,
    _: JClass,
    r: jlong,
) -> jint {
    renderer_layout(r as *const Renderer) as jint
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nSkinBoneCount(
    _: JNIEnv,
    _: JClass,
    r: jlong,
) -> jint {
    renderer_skin_bone_count(r as *const Renderer) as jint
}

#[no_mangle]
pub extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nSkinPositions(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
) -> jfloatArray {
    floats(&mut env, &scene(r).skin.positions)
}

#[no_mangle]
pub extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nSkinNormals(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
) -> jfloatArray {
    floats(&mut env, &scene(r).skin.normals)
}

#[no_mangle]
pub extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nSkinUvs(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
) -> jfloatArray {
    floats(&mut env, &scene(r).skin.uvs)
}

#[no_mangle]
pub extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nSkinGlow(
    env: JNIEnv,
    _: JClass,
    r: jlong,
) -> jbyteArray {
    env.byte_array_from_slice(&scene(r).skin.glow)
        .map_or(std::ptr::null_mut(), |a| a.into_raw())
}

#[no_mangle]
pub extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nSkinBones(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
) -> jintArray {
    ints(&mut env, words(&scene(r).skin.bones))
}

#[no_mangle]
pub extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nSkinRuns(
    mut env: JNIEnv,
    _: JClass,
    r: jlong,
) -> jintArray {
    ints(&mut env, words(&scene(r).skin.runs))
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_polyfrost_plusrenderer_Renderer_nFrame(
    env: JNIEnv,
    _: JClass,
    r: jlong,
    seconds: jfloat,
    input: JFloatArray,
    part_count: jint,
    query_count: jint,
    space: jint,
    out: JFloatArray,
) -> jint {
    let renderer = &mut *(r as *mut Renderer);
    let (parts, queries) = (part_count.max(0) as usize * 12, query_count.max(0) as usize);

    renderer.input.resize(parts + queries, 0.0);
    if env
        .get_float_array_region(&input, 0, &mut renderer.input)
        .is_err()
    {
        return fail(
            renderer,
            "input holds fewer floats than its parts and queries",
        );
    }
    let input = std::mem::take(&mut renderer.input);
    let bones = renderer_frame(
        renderer,
        seconds,
        input.as_ptr(),
        part_count.max(0) as u32,
        input[parts..].as_ptr(),
        query_count.max(0) as u32,
        space as u32,
    );
    renderer.input = input;
    if bones < 0 {
        return bones;
    }

    let scene = &renderer.scene;
    let needed = scene.skin_transforms.len() + scene.skin_offsets.len();
    let held = env.get_array_length(&out).unwrap_or(0) as usize;
    if held < needed {
        return fail(
            renderer,
            &format!("out holds {held} floats, the frame needs {needed}"),
        );
    }
    let written = env
        .set_float_array_region(&out, 0, &scene.skin_transforms)
        .is_ok()
        && env
            .set_float_array_region(
                &out,
                scene.skin_transforms.len() as i32,
                &scene.skin_offsets,
            )
            .is_ok();
    if written {
        bones
    } else {
        fail(renderer, "could not write the frame")
    }
}
