use std::sync::Arc;

use crate::colour::Colour;
use crate::fragment::Fragment;
use crate::material::MaterialCollisionInfo;
use crate::scene::Scene;

pub struct SampleLighting {
  pub diffuse: Colour,
  pub ambient: Colour,
  pub specular: Colour,
}

pub trait LightingIntegrator: Sync + Send {
  fn lighting(&self, configuration: &Scene, fragment: &Fragment, surface: &MaterialCollisionInfo) -> SampleLighting;
}

pub struct RenderConfiguration {
  lighting_integrator: Arc<LightingIntegrator>,
  scene: Arc<Scene>,
}

impl RenderConfiguration {
  pub fn new(lighting_integrator: Arc<LightingIntegrator>, scene: Arc<Scene>) -> Self {
    return Self {
      lighting_integrator,
      scene,
    };
  }

  pub fn scene(&self) -> Arc<Scene> {
    return self.scene.clone();
  }

  pub fn lighting_integrator(&self) -> Arc<LightingIntegrator> {
    return self.lighting_integrator.clone();
  }
}
