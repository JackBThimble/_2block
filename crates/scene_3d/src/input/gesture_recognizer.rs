//! Low-level touch gesture recognition
//! Detects: single tap, double tap, drag, pinch, two-finger pan

use bevy::input::touch::Touches;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct TouchPoint {
    pub position: Vec2,
    pub start_position: Vec2,
    pub start_time: f32,
    pub previous_position: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GestureType {
    None,
    SingleFingerDrag,
    TwoFingerPan,
    Pinch,
    Tap,
    DoubleTap,
}

#[derive(Resource)]
pub struct GestureState {
    pub active_touches: HashMap<u64, TouchPoint>,
    pub current_gesture: GestureType,

    // Pinch tracking
    pub initial_pinch_distance: Option<f32>,
    pub last_pinch_distance: Option<f32>,

    // Pan tracking
    pub last_two_finger_center: Option<Vec2>,

    // Tap detection
    pub last_tap_time: f32,
    pub last_tap_position: Option<Vec2>,

    // Constants
    pub tap_max_duration: f32,
    pub tap_max_movement: f32,
    pub double_tap_max_interval: f32,
    pub double_tap_max_distance: f32,
}

impl Default for GestureState {
    fn default() -> Self {
        Self {
            active_touches: HashMap::new(),
            current_gesture: GestureType::None,
            initial_pinch_distance: None,
            last_pinch_distance: None,
            last_two_finger_center: None,
            last_tap_time: 0.0,
            last_tap_position: None,
            tap_max_duration: 0.2,
            tap_max_movement: 10.0,
            double_tap_max_interval: 0.3,
            double_tap_max_distance: 50.0,
        }
    }
}

pub struct RecognizedGesture {
    pub gesture_type: GestureType,
    pub position: Vec2,
    pub delta: Vec2,
    pub pinch_delta: Option<f32>,
}

impl GestureState {
    /// Update gesture state with current touch input
    /// Returns recognized gesture if any
    pub fn update(&mut self, touches: &Touches, current_time: f32) -> Option<RecognizedGesture> {
        for touch in touches.iter_just_pressed() {
            self.active_touches.insert(
                touch.id(),
                TouchPoint {
                    position: touch.position(),
                    start_position: touch.start_position(),
                    start_time: current_time,
                    previous_position: touch.previous_position(),
                },
            );
        }

        for touch in touches.iter() {
            if let Some(point) = self.active_touches.get_mut(&touch.id()) {
                point.previous_position = point.position;
                point.position = touch.position();
            }
        }

        for touch in touches.iter_just_released() {
            if let Some(point) = self.active_touches.remove(&touch.id()) {
                // Check if this was a tap
                let duration = current_time - point.start_time;
                let movement = point.position.distance(point.start_position);

                if duration < self.tap_max_duration && movement < self.tap_max_movement {
                    return self.handle_tap(point.position, current_time);
                }
            }
        }

        // ====================================================================
        // HANDLE CANCELED TOUCHES
        // ====================================================================
        for touch in touches.iter_just_canceled() {
            self.active_touches.remove(&touch.id());
        }

        // Reset gesture state when no touches are active
        if self.active_touches.is_empty() {
            self.current_gesture = GestureType::None;
            self.initial_pinch_distance = None;
            self.last_pinch_distance = None;
            self.last_two_finger_center = None;
        }

        // Recognize ongoing gestures
        self.recognize_gesture(current_time)
    }

    fn handle_tap(&mut self, position: Vec2, current_time: f32) -> Option<RecognizedGesture> {
        // Check for double tap
        if let Some(last_pos) = self.last_tap_position {
            let time_since_last = current_time - self.last_tap_time;
            let distance = position.distance(last_pos);

            if time_since_last < self.double_tap_max_interval
                && distance < self.double_tap_max_distance
            {
                // Double tap detected!
                self.last_tap_time = 0.0;
                self.last_tap_position = None;

                return Some(RecognizedGesture {
                    gesture_type: GestureType::DoubleTap,
                    position,
                    delta: Vec2::ZERO,
                    pinch_delta: None,
                });
            }
        }

        // Record this tap for potential double tap
        self.last_tap_time = current_time;
        self.last_tap_position = Some(position);

        Some(RecognizedGesture {
            gesture_type: GestureType::Tap,
            position,
            delta: Vec2::ZERO,
            pinch_delta: None,
        })
    }

    fn recognize_gesture(&mut self, _current_time: f32) -> Option<RecognizedGesture> {
        let touch_count = self.active_touches.len();

        match touch_count {
            0 => {
                self.current_gesture = GestureType::None;
                None
            }

            1 => {
                // Single finger drag
                self.current_gesture = GestureType::SingleFingerDrag;

                let point = self.active_touches.values().next().unwrap();
                let delta = point.position - point.previous_position;

                if delta.length_squared() > 0.1 {
                    Some(RecognizedGesture {
                        gesture_type: GestureType::SingleFingerDrag,
                        position: point.position,
                        delta,
                        pinch_delta: None,
                    })
                } else {
                    None
                }
            }

            2 => {
                // Two finger: detect pinch and pan
                let points: Vec<&TouchPoint> = self.active_touches.values().collect();
                let p0 = points[0];
                let p1 = points[1];

                let center = (p0.position + p1.position) / 2.0;
                let current_distance = p0.position.distance(p1.position);

                // Initialize pinch tracking
                if self.initial_pinch_distance.is_none() {
                    self.initial_pinch_distance = Some(current_distance);
                    self.last_pinch_distance = Some(current_distance);
                }

                let mut result: Option<RecognizedGesture> = None;

                // Detect pinch
                if let Some(last_distance) = self.last_pinch_distance {
                    let distance_delta = current_distance - last_distance;

                    if distance_delta.abs() > 1.0 {
                        self.current_gesture = GestureType::Pinch;
                        result = Some(RecognizedGesture {
                            gesture_type: GestureType::Pinch,
                            position: center,
                            delta: Vec2::ZERO,
                            pinch_delta: Some(distance_delta),
                        });
                    }
                }

                // Detect two-finger pan
                if let Some(last_center) = self.last_two_finger_center {
                    let center_delta = center - last_center;

                    if center_delta.length_squared() > 1.0 {
                        if result.is_none() {
                            self.current_gesture = GestureType::TwoFingerPan;
                            result = Some(RecognizedGesture {
                                gesture_type: GestureType::TwoFingerPan,
                                position: center,
                                delta: center_delta,
                                pinch_delta: None,
                            });
                        } else if let Some(ref mut gesture) = result {
                            // Combine pinch and pan
                            gesture.delta = center_delta;
                        }
                    }
                }

                // Update tracking
                self.last_pinch_distance = Some(current_distance);
                self.last_two_finger_center = Some(center);

                result
            }

            _ => {
                // Three+ fingers: could add rotate or other gestures
                None
            }
        }
    }
}
