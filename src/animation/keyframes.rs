use crate::model::Channel;

pub enum Keyframes {
    Translation(Vec<[f32; 3]>),
    Rotation(Vec<[f32; 4]>),
    Scale(Vec<[f32; 3]>),
    Weights(Vec<f32>), // probably wrong
    Empty
}

pub fn interpolate_position(channel: &Channel, time: f32) -> Option<[f32; 3]> {
    let timestamps = &channel.timestamps;
    let frames = match &channel.keyframes {
        Keyframes::Translation(v) => v,
        _ => return None
    };

    if timestamps.is_empty() {
        return None;
    }

    // find the frame interval
    let i = match timestamps.binary_search_by(|probe| probe.partial_cmp(&time).unwrap()) {
        Ok(idx) => idx,
        Err(idx) => if idx == 0 { 0 } else { idx - 1 },
    };
    
    let t0 = timestamps[i];
    let t1 = if i + 1 < timestamps.len() { timestamps[i + 1] } else { t0 };
    let factor = if t1 != t0 { (time - t0) / (t1 - t0) } else { 0.0 };

   
    let start = frames[i];
    let end = if i + 1 < frames.len() { frames[i + 1] } else { start };

    Some([
        start[0] + (end[0] - start[0]) * factor,
        start[1] + (end[1] - start[1]) * factor,
        start[2] + (end[2] - start[2]) * factor,
    ])
}

pub fn interpolate_rotation(channel: &Channel, time: f32) -> Option<[f32; 4]> {
    let timestamps = &channel.timestamps;
    let frames = match &channel.keyframes {
        Keyframes::Rotation(v) => v,
        _ => return None
    };

    if timestamps.is_empty() {
        return None;
    }

    let i = match timestamps.binary_search_by(|probe| probe.partial_cmp(&time).unwrap()) {
        Ok(idx) => idx,
        Err(idx) => if idx == 0 { 0 } else { idx - 1 },
    };

    let t0 = timestamps[i];
    let t1 = if i + 1 < timestamps.len() { timestamps[i + 1] } else { t0 };
    let factor = if t1 != t0 { (time - t0) / (t1 - t0) } else { 0.0 };

    let start = cgmath::Quaternion::from(frames[i]);
    let end = cgmath::Quaternion::from(if i + 1 < frames.len() { frames[i + 1] } else { frames[i] });

    let interp = start.slerp(end, factor);
    Some(interp.into())
}

pub fn interpolate_scale(channel: &Channel, time: f32) -> Option<[f32; 3]> {
    let timestamps = &channel.timestamps;
    let frames = match &channel.keyframes {
        Keyframes::Rotation(v) => v,
        _ => return None
    };

    if timestamps.is_empty() {
        return None;
    }

    let i = match timestamps.binary_search_by(|probe| probe.partial_cmp(&time).unwrap()) {
        Ok(idx) => idx,
        Err(idx) => if idx == 0 { 0 } else { idx - 1 },
    };

    let t0 = timestamps[i];
    let t1 = if i + 1 < timestamps.len() { timestamps[i + 1] } else { t0 };
    let factor = if t1 != t0 { (time - t0) / (t1 - t0) } else { 0.0 };

    let start = frames[i];
    let end = if i + 1 < frames.len() { frames[i + 1] } else { start };

    Some([
        start[0] + (end[0] - start[0]) * factor,
        start[1] + (end[1] - start[1]) * factor,
        start[2] + (end[2] - start[2]) * factor,
    ])
}