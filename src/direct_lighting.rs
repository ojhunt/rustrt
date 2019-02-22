use crate::photon_map::random;
use crate::scene::Scene;
use crate::light::LightSample;
use crate::material::MaterialCollisionInfo;
use std::sync::Arc;
use crate::render_configuration::SampleLighting;
use crate::colour::Colour;
use crate::fragment::Fragment;
use crate::render_configuration::LightingIntegrator;
use crate::ray::Ray;
use crate::vectors::Vector;

pub trait IndirectLightingSource: Sync + Send {
  fn lighting_and_shadow(
    &self,
    scene: &Scene,
    fragment: &Fragment,
    surface: &MaterialCollisionInfo,
  ) -> (Option<Colour>, Option<bool>);
}

pub struct DirectLighting {
  indirect_lighting: Option<Arc<IndirectLightingSource>>,
  lights: Arc<Vec<LightSample>>,
}

impl DirectLighting {
  pub fn new(_: &Arc<Scene>, lights: Vec<LightSample>, indirect_lighting: Option<Arc<IndirectLightingSource>>) -> Self {
    return DirectLighting {
      indirect_lighting,
      lights: Arc::new(lights),
    };
  }
}

impl LightingIntegrator for DirectLighting {
  fn lighting(&self, scene: &Scene, fragment: &Fragment, surface: &MaterialCollisionInfo) -> SampleLighting {
    let (photon_lighting, had_shadow) = if let Some(ref photon_map) = self.indirect_lighting {
      photon_map.lighting_and_shadow(scene, fragment, surface)
    } else {
      (None, None)
    };

    let light_samples = 50;
    let lights = &*self.lights;
    let light_scale = lights.len() as f32 / light_samples as f32;
    let mut diffuse_lighting = Vector::new();
    let mut ambient_lighting = Vector::new();
    let mut _specular_lighting = Vector::new();

    for _ in 0..light_samples {
      let light = &lights[random(0.0, lights.len() as f64) as usize];
      let mut ldir = light.position - surface.position;
      let ldir_len = ldir.length();
      ldir = ldir.normalize();
      if had_shadow.unwrap_or(true) {
        let shadow_test = Ray::new_bound(surface.position, ldir, 0.02, ldir_len - 0.001, None);
        if scene.intersect(&shadow_test).is_some() {
          continue;
        }
      }

      let diffuse_intensity = light_scale * light.weight * ldir.dot(surface.normal).max(0.0);
      let ambient_intensity = light_scale * light.weight * light.ambient;
      diffuse_lighting = diffuse_lighting + light.diffuse * diffuse_intensity;
      ambient_lighting = ambient_lighting + light.ambient * ambient_intensity;
    }

    return SampleLighting {
      diffuse: Colour::from(diffuse_lighting),
      ambient: photon_lighting.unwrap_or(Colour::from(ambient_lighting)),
      specular: Colour::new(),
    };
  }
}
