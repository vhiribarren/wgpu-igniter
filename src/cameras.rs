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

use cgmath::{Matrix4, PerspectiveFov, Rad, Vector3, vec3};
use cgmath::{Ortho, Point3};
use log::{debug, warn};
use std::collections::BTreeSet;
use std::f32::consts::PI;
use std::sync::LazyLock;
use winit::event::{DeviceEvent, ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

static SWITCH_Z_AXIS: LazyLock<Matrix4<f32>> =
    LazyLock::new(|| Matrix4::from_nonuniform_scale(1., 1., -1.));
static TO_WEBGPU_NDCS: LazyLock<Matrix4<f32>> = LazyLock::new(|| {
    Matrix4::from_translation(vec3(0., 0., 0.5)) * Matrix4::from_nonuniform_scale(1., 1., 0.5)
});

pub struct OrthogonalConfig {
    pub width: f32,
    pub height: f32,
    pub eye: Point3<f32>,
    pub center: Point3<f32>,
    pub up: Vector3<f32>,
    pub near: f32,
    pub far: f32,
}

impl Default for OrthogonalConfig {
    fn default() -> Self {
        OrthogonalConfig {
            width: 16.0 / 4.0,
            height: 9.0 / 4.0,
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
            near: 0.,
            far: 1_000.0,
        }
    }
}

impl From<OrthogonalConfig> for Camera {
    fn from(config: OrthogonalConfig) -> Self {
        Camera {
            projection: Matrix4::from(Ortho {
                left: -config.width / 2.0,
                right: config.width / 2.0,
                bottom: -config.height / 2.0,
                top: config.height / 2.0,
                near: config.near,
                far: config.far,
            }),
            view: Matrix4::look_at_lh(config.eye, config.center, config.up),
        }
    }
}

pub struct PerspectiveConfig {
    pub fovy: f32,
    pub aspect: f32,
    pub eye: Point3<f32>,
    pub center: Point3<f32>,
    pub up: Vector3<f32>,
    pub near: f32,
    pub far: f32,
}

impl Default for PerspectiveConfig {
    fn default() -> Self {
        PerspectiveConfig {
            fovy: PI / 4.0,
            aspect: 16. / 9.,
            near: 0.1,
            far: 1_000.0,
            eye: Point3 {
                x: 0.0,
                y: 0.0,
                z: -5.0,
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

impl From<PerspectiveConfig> for Camera {
    fn from(config: PerspectiveConfig) -> Self {
        Camera {
            projection: Matrix4::from(PerspectiveFov {
                fovy: Rad(config.fovy),
                aspect: config.aspect,
                near: config.near,
                far: config.far,
            }),
            view: Matrix4::look_at_lh(config.eye, config.center, config.up),
        }
    }
}

#[derive(Debug)]
pub struct Camera {
    pub projection: Matrix4<f32>,
    pub view: Matrix4<f32>,
}

impl Camera {
    pub fn get_camera_matrix(&self) -> Matrix4<f32> {
        (*TO_WEBGPU_NDCS) * self.projection * (*SWITCH_Z_AXIS) * self.view
    }
    fn move_z(&mut self, val: f32) {
        self.view = Matrix4::from_translation(Vector3::new(0., 0., -val)) * self.view;
    }
    fn move_x(&mut self, val: f32) {
        self.view = Matrix4::from_translation(Vector3::new(-val, 0., 0.)) * self.view;
    }
    fn move_y(&mut self, val: f32) {
        self.view = Matrix4::from_translation(Vector3::new(0., -val, 0.)) * self.view;
    }
    fn pan(&mut self, val: f32) {
        self.view = Matrix4::from_angle_y(Rad(-val)) * self.view;
    }
    fn tilt(&mut self, val: f32) {
        self.view = Matrix4::from_angle_x(Rad(-val)) * self.view;
    }
    #[allow(dead_code)]
    fn roll(&mut self, val: f32) {
        self.view = Matrix4::from_angle_z(Rad(-val)) * self.view;
    }
}

pub struct WinitCameraAdapter {
    camera: Camera,
    enabled_keys: BTreeSet<KeyCode>,
    key_speed: f32,
    rotation_speed: f32,
}

impl WinitCameraAdapter {
    const DEFAULT_KEY_SPEED: f32 = 0.03;
    const DEFAULT_ROTATION_SPEED: f32 = 1.0 / 500.0;

    pub fn new(camera: Camera) -> Self {
        WinitCameraAdapter {
            camera,
            enabled_keys: BTreeSet::new(),
            key_speed: Self::DEFAULT_KEY_SPEED,
            rotation_speed: Self::DEFAULT_ROTATION_SPEED,
        }
    }

    pub fn get_camera_matrix(&self) -> Matrix4<f32> {
        self.camera.get_camera_matrix()
    }

    pub fn mouse_event_listener(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.camera.pan(delta.0 as f32 * self.rotation_speed);
                self.camera.tilt(delta.1 as f32 * self.rotation_speed);
            }
            DeviceEvent::MouseWheel {
                delta: _scroll_delta,
            } => {}
            _ => {}
        };
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

    pub fn update(&mut self) {
        if self.enabled_keys.is_empty() {
            return;
        }
        for key in self.enabled_keys.iter() {
            match *key {
                KeyCode::ArrowUp => self.camera.move_z(self.key_speed),
                KeyCode::ArrowDown => self.camera.move_z(-self.key_speed),
                KeyCode::ArrowLeft => self.camera.move_x(-self.key_speed),
                KeyCode::ArrowRight => self.camera.move_x(self.key_speed),
                KeyCode::PageUp => self.camera.move_y(self.key_speed),
                KeyCode::PageDown => self.camera.move_y(-self.key_speed),
                _ => {}
            };
        }
        debug!("{:?}", -self.as_ref().view);
    }
}

impl AsRef<Camera> for WinitCameraAdapter {
    fn as_ref(&self) -> &Camera {
        &self.camera
    }
}
