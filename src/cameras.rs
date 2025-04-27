/*
MIT License

Copyright (c) 2021, 2022, 2024, 2025 Vincent Hiribarren

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use cgmath::{InnerSpace, Matrix3, Matrix4, PerspectiveFov, Rad, Vector3, vec3};
use cgmath::{Ortho, Point3};
use log::warn;
use std::collections::BTreeSet;
use std::f32::consts::PI;
use std::sync::LazyLock;
use winit::event::{DeviceEvent, ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::Dimensions;

static SWITCH_Z_AXIS: LazyLock<Matrix4<f32>> =
    LazyLock::new(|| Matrix4::from_nonuniform_scale(1., 1., -1.));
static TO_WEBGPU_NDCS: LazyLock<Matrix4<f32>> = LazyLock::new(|| {
    Matrix4::from_translation(vec3(0., 0., 0.5)) * Matrix4::from_nonuniform_scale(1., 1., 0.5)
});

pub struct CameraView {
    pub eye: Point3<f32>,
    pub center: Point3<f32>,
    pub up: Vector3<f32>,
}

impl CameraView {
    #[must_use]
    pub fn calc_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_lh(self.eye, self.center, self.up)
    }
    pub fn move_x(&mut self, val: f32, lock_center: bool) {
        let right = self.up.cross((self.center - self.eye).normalize());
        self.eye += right * val;
        if !lock_center {
            self.center += right * val;
        }
    }
    pub fn move_y(&mut self, val: f32, lock_center: bool) {
        self.eye += self.up * val;
        if !lock_center {
            self.center += self.up * val;
        }
    }
    pub fn move_z(&mut self, val: f32, lock_center: bool) {
        let forward = (self.center - self.eye).normalize();
        self.eye += forward * val;
        if !lock_center {
            self.center += forward * val;
        }
    }
    pub fn roll(&mut self, val: f32) {
        let forward = (self.center - self.eye).normalize();
        let rotation = Matrix3::from_axis_angle(forward, Rad(val));
        self.up = rotation * self.up;
    }
    pub fn tilt(&mut self, val: f32) {
        let forward = (self.center - self.eye).normalize();
        let right = self.up.cross(forward);
        let rotation = Matrix3::from_axis_angle(right, Rad(val));
        self.up = rotation * self.up;
        let rotated_forward = Matrix3::from_axis_angle(self.up.cross(forward), Rad(val)) * forward;
        self.center = self.eye + rotated_forward * (self.center - self.eye).magnitude();
    }
    pub fn pan(&mut self, val: f32) {
        let forward = (self.center - self.eye).normalize();
        let rotation = Matrix3::from_axis_angle(self.up, Rad(val));
        let rotated_forward = rotation * forward;
        self.center = self.eye + rotated_forward * (self.center - self.eye).magnitude();
    }
}

impl Default for CameraView {
    fn default() -> Self {
        Self {
            eye: Point3 {
                x: 0.0,
                y: 0.0,
                z: -10.0,
            },
            center: Point3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            up: Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        }
    }
}

pub trait CameraProjection {
    fn calc_projection(&self) -> Matrix4<f32>;
    fn resize_screen(&mut self, dimensions: Dimensions);
}

pub struct OrthogonalCameraConfig {
    pub width: f32,
    pub height: f32,
    pub near: f32,
    pub far: f32,
}

impl CameraProjection for OrthogonalCameraConfig {
    fn calc_projection(&self) -> Matrix4<f32> {
        Matrix4::from(Ortho {
            left: -self.width / 2.0,
            right: self.width / 2.0,
            bottom: -self.height / 2.0,
            top: self.height / 2.0,
            near: self.near,
            far: self.far,
        })
    }
    #[allow(clippy::cast_precision_loss)]
    fn resize_screen(&mut self, dimensions: Dimensions) {
        self.width = dimensions.width as f32;
        self.height = dimensions.height as f32;
    }
}

impl Default for OrthogonalCameraConfig {
    fn default() -> Self {
        Self {
            width: 16.0 / 4.0,
            height: 9.0 / 4.0,
            near: 0.,
            far: 1_000.0,
        }
    }
}

pub struct PerspectiveCameraConfig {
    pub fovy: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for PerspectiveCameraConfig {
    fn default() -> Self {
        Self {
            fovy: PI / 4.0,
            aspect: 16. / 9.,
            near: 0.1,
            far: 1_000.0,
        }
    }
}

impl CameraProjection for PerspectiveCameraConfig {
    fn calc_projection(&self) -> Matrix4<f32> {
        Matrix4::from(PerspectiveFov {
            fovy: Rad(self.fovy),
            aspect: self.aspect,
            near: self.near,
            far: self.far,
        })
    }
    #[allow(clippy::cast_precision_loss)]
    fn resize_screen(&mut self, dimensions: Dimensions) {
        self.aspect = dimensions.width as f32 / dimensions.height as f32;
    }
}

pub struct Camera {
    projection: Box<dyn CameraProjection>,
    view: CameraView,
    projection_cache: Matrix4<f32>,
    view_cache: Matrix4<f32>,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(
            CameraView::default(),
            Box::new(PerspectiveCameraConfig::default()),
        )
    }
}
// TODO Provide method to replace the project and the view directly
impl Camera {
    #[must_use]
    pub fn new(view: CameraView, projection: Box<dyn CameraProjection>) -> Self {
        let view_cache = view.calc_view_matrix();
        let projection_cache = projection.calc_projection();
        Self {
            projection,
            view,
            projection_cache,
            view_cache,
        }
    }
    fn update_view_cache(&mut self) {
        self.view_cache = self.view.calc_view_matrix();
    }
    fn update_projection_cache(&mut self) {
        self.projection_cache = self.projection.calc_projection();
    }
    pub fn resize_screen(&mut self, dimensions: Dimensions) {
        self.projection.resize_screen(dimensions);
        self.update_projection_cache();
    }
    #[must_use]
    pub fn get_camera_matrix(&self) -> Matrix4<f32> {
        (*TO_WEBGPU_NDCS) * self.projection_cache * (*SWITCH_Z_AXIS) * self.view_cache
    }
    #[must_use]
    pub fn eye_position(&self) -> Point3<f32> {
        self.view.eye
    }
    pub fn move_z(&mut self, val: f32) {
        self.view.move_z(val, false);
        self.update_view_cache();
    }
    pub fn move_x(&mut self, val: f32) {
        self.view.move_x(val, false);
        self.update_view_cache();
    }
    pub fn move_y(&mut self, val: f32) {
        self.view.move_y(val, false);
        self.update_view_cache();
    }
    pub fn pan(&mut self, val: f32) {
        self.view.pan(val);
        self.update_view_cache();
    }
    pub fn tilt(&mut self, val: f32) {
        self.view.tilt(val);
        self.update_view_cache();
    }
    pub fn roll(&mut self, val: f32) {
        self.view.roll(val);
        self.update_view_cache();
    }
}

pub struct InteractiveCamera {
    pub controled_camera: Camera,
    enabled_keys: BTreeSet<KeyCode>,
    key_speed: f32,
    rotation_speed: f32,
}

impl InteractiveCamera {
    const DEFAULT_KEY_SPEED: f32 = 0.03;
    const DEFAULT_ROTATION_SPEED: f32 = 1.0 / 500.0;
    const SPEED_MULTIPLICATOR: f32 = 10.0;

    #[must_use]
    pub fn new(camera: Camera) -> Self {
        Self {
            controled_camera: camera,
            enabled_keys: BTreeSet::new(),
            key_speed: Self::DEFAULT_KEY_SPEED,
            rotation_speed: Self::DEFAULT_ROTATION_SPEED,
        }
    }

    #[must_use]
    pub fn get_camera_matrix(&self) -> Matrix4<f32> {
        self.controled_camera.get_camera_matrix()
    }

    pub fn update_screen_size(&mut self, dimensions: Dimensions) {
        self.controled_camera.resize_screen(dimensions);
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn mouse_event_listener(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.controled_camera
                    .pan(delta.0 as f32 * self.rotation_speed);
                self.controled_camera
                    .tilt(delta.1 as f32 * self.rotation_speed);
            }
            DeviceEvent::MouseWheel {
                delta: _scroll_delta,
            } => {}
            _ => {}
        }
    }

    pub fn keyboard_event_listener(&mut self, input: &KeyEvent) {
        let PhysicalKey::Code(key_code) = input.physical_key else {
            warn!("Strange key pushed");
            return;
        };
        if input.state == ElementState::Pressed {
            self.enabled_keys.insert(key_code);
        } else {
            self.enabled_keys.remove(&key_code);
        }
    }

    pub fn update_control(&mut self) {
        if self.enabled_keys.is_empty() {
            return;
        }
        let mut key_speed = self.key_speed;
        if self.enabled_keys.contains(&KeyCode::ShiftLeft)
            || self.enabled_keys.contains(&KeyCode::ShiftRight)
        {
            key_speed *= Self::SPEED_MULTIPLICATOR;
        }
        for key in &self.enabled_keys {
            match *key {
                KeyCode::ArrowUp => self.controled_camera.move_z(key_speed),
                KeyCode::ArrowDown => self.controled_camera.move_z(-key_speed),
                KeyCode::ArrowLeft => self.controled_camera.move_x(-key_speed),
                KeyCode::ArrowRight => self.controled_camera.move_x(key_speed),
                KeyCode::PageUp => self.controled_camera.move_y(key_speed),
                KeyCode::PageDown => self.controled_camera.move_y(-key_speed),
                KeyCode::Home => self.controled_camera.roll(-key_speed / 2.0),
                KeyCode::End => self.controled_camera.roll(key_speed / 2.0),
                _ => {}
            }
        }
    }
}

impl AsRef<Camera> for InteractiveCamera {
    fn as_ref(&self) -> &Camera {
        &self.controled_camera
    }
}
