#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToggleId {
    Bloom,
    Vignette,
    DreamAtmosphere,
    AmberColorGrade,
    AmbientOrbs,
    AmbientFill,
    Particles,
    Breathing,
    ScreenPostfx,
}

pub struct EffectsConfig {
    pub bloom: bool,
    pub vignette: bool,
    pub dream_atmosphere: bool,
    pub amber_color_grade: bool,
    pub ambient_orbs: bool,
    pub ambient_fill: bool,
    pub particles: bool,
    pub breathing: bool,
    pub screen_postfx: bool,
}

impl Default for EffectsConfig {
    fn default() -> Self {
        Self {
            bloom: true,
            vignette: true,
            dream_atmosphere: false,
            amber_color_grade: true,
            ambient_orbs: false,
            ambient_fill: false,
            particles: false,
            breathing: true,
            screen_postfx: true,
        }
    }
}

impl EffectsConfig {
    pub fn get(&self, id: ToggleId) -> bool {
        match id {
            ToggleId::Bloom => self.bloom,
            ToggleId::Vignette => self.vignette,
            ToggleId::DreamAtmosphere => self.dream_atmosphere,
            ToggleId::AmberColorGrade => self.amber_color_grade,
            ToggleId::AmbientOrbs => self.ambient_orbs,
            ToggleId::AmbientFill => self.ambient_fill,
            ToggleId::Particles => self.particles,
            ToggleId::Breathing => self.breathing,
            ToggleId::ScreenPostfx => self.screen_postfx,
        }
    }

    pub fn set(&mut self, id: ToggleId, value: bool) {
        match id {
            ToggleId::Bloom => self.bloom = value,
            ToggleId::Vignette => self.vignette = value,
            ToggleId::DreamAtmosphere => self.dream_atmosphere = value,
            ToggleId::AmberColorGrade => self.amber_color_grade = value,
            ToggleId::AmbientOrbs => self.ambient_orbs = value,
            ToggleId::AmbientFill => self.ambient_fill = value,
            ToggleId::Particles => self.particles = value,
            ToggleId::Breathing => self.breathing = value,
            ToggleId::ScreenPostfx => self.screen_postfx = value,
        }
    }

    pub fn toggle(&mut self, id: ToggleId) {
        let current = self.get(id);
        self.set(id, !current);
    }

    /// Minimal config that disables all expensive per-frame effects.
    /// Used when agents are actively running to reduce CPU overhead.
    pub fn degraded() -> Self {
        Self {
            bloom: false,
            vignette: false,
            dream_atmosphere: false,
            amber_color_grade: false,
            ambient_orbs: false,
            ambient_fill: false,
            particles: false,
            breathing: false,
            screen_postfx: false,
        }
    }
}
