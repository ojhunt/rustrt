use crate::vectors::Vector;
use crate::light::LightSample;
use crate::camera::Camera;
use crate::scene::Scene;
use std::sync::Arc;
use crate::material::MaterialCollisionInfo;
use crate::fragment::Fragment;
use crate::colour::Colour;

pub struct SampleLighting {
  pub diffuse: Colour,
  pub ambient: Colour,
  pub specular: Colour,
}

impl SampleLighting {
  pub fn new() -> Self {
    SampleLighting {
      ambient: Colour::new(),
      diffuse: Colour::new(),
      specular: Colour::new(),
    }
  }
}
pub trait LightingIntegrator: Sync + Send {
  fn lighting(&self, configuration: &Scene, fragment: &Fragment, surface: &MaterialCollisionInfo) -> SampleLighting;
}

pub struct RenderConfiguration {
  lighting_integrator: Arc<Box<LightingIntegrator>>,
  scene: Arc<Scene>,
  camera: Arc<Box<Camera>>,
  lights: Arc<Vec<LightSample>>,
}

impl RenderConfiguration {
  pub fn new(lighting_integrator: Arc<Box<LightingIntegrator>>, scene: Arc<Scene>, camera: Arc<Box<Camera>>) -> Self {
    let lights = Arc::new(scene.get_light_samples(10000));
    return Self {
      lighting_integrator,
      scene,
      camera,
      lights: lights,
    };
  }
  pub fn camera(&self) -> Arc<Box<Camera>> {
    return self.camera.clone();
  }

  pub fn scene(&self) -> Arc<Scene> {
    return self.scene.clone();
  }

  pub fn lighting_integrator(&self) -> Arc<Box<LightingIntegrator>> {
    return self.lighting_integrator.clone();
  }

  pub fn lights(&self) -> Arc<Vec<LightSample>> {
    return self.lights.clone();
  }
}
