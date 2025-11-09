use shogi_engine::evaluation::phase_transition::{
    InterpolationMethod, PhaseTransition, PhaseTransitionConfig,
};
use shogi_engine::types::TaperedScore;

fn create_transition(method: InterpolationMethod) -> PhaseTransition {
    if method == InterpolationMethod::Advanced {
        let mut config = PhaseTransitionConfig::default();
        config.use_advanced_interpolator = true;
        config.default_method = InterpolationMethod::Advanced;
        PhaseTransition::with_config(config)
    } else {
        PhaseTransition::new()
    }
}

#[test]
fn test_interpolation_methods_across_phase_checkpoints() {
    let phases = [0, 64, 128, 192, 256];
    let base_score = TaperedScore::new_tapered(200, 100);

    for method in [
        InterpolationMethod::Linear,
        InterpolationMethod::Cubic,
        InterpolationMethod::Sigmoid,
        InterpolationMethod::Smoothstep,
        InterpolationMethod::Advanced,
    ] {
        let mut transition = create_transition(method);
        let mut values = Vec::new();

        for phase in phases {
            let value = transition.interpolate(base_score, phase, method);
            assert!(
                (100..=200).contains(&value),
                "Interpolated value {:?} should stay within score bounds for method {:?}",
                value,
                method
            );
            values.push(value);
        }

        if method != InterpolationMethod::Advanced {
            for window in values.windows(2) {
                let (prev, next) = (window[0], window[1]);
                assert!(
                    prev <= next + 2,
                    "Interpolation for {:?} should remain monotonic: {} -> {}",
                    method,
                    prev,
                    next
                );
            }
        }

        if method != InterpolationMethod::Advanced {
            assert!(
                transition.validate_smooth_transitions(base_score, method),
                "Smoothness validation should pass for method {:?}",
                method
            );
        }
    }
}
