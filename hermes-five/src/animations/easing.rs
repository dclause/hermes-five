use simple_easing::*;

/// Represents a set of easing function.
///
/// An easing function is a temporal function that takes a time between 0 and 1 (beginning / end)
/// and associate to it a number value according to an ease curve.
///
/// See <https://easings.net> for a representation of easing methods.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    /// <https://easings.net/#easeInBack>
    BackIn,
    /// <https://easings.net/#easeInOutBack>
    BackInOuT,
    /// <https://easings.net/#easeOutBack>
    BackOut,
    /// <https://easings.net/#easeInBounce>
    BounceIn,
    /// <https://easings.net/#easeInOutBounce>
    BounceInOut,
    /// <https://easings.net/#easeOutBounce>
    BounceOut,
    /// <https://easings.net/#easeInCirc>
    CircIn,
    /// <https://easings.net/#easeInOutCirc>
    CircInOut,
    /// <https://easings.net/#easeOutCirc>
    CircOut,
    /// <https://easings.net/#easeInCubic>
    CubicIn,
    /// <https://easings.net/#easeInOutCubic>
    CubicInOut,
    /// <https://easings.net/#easeOutCubic>
    CubicOut,
    /// <https://easings.net/#easeInElastic>
    ElasticIn,
    /// <https://easings.net/#easeInOutElastic>
    ElasticInOut,
    /// <https://easings.net/#easeOutElastic>
    ElasticOut,
    /// <https://easings.net/#easeInExpo>
    ExpoIn,
    /// <https://easings.net/#easeInOutExpo>
    ExpoInOut,
    /// <https://easings.net/#easeOutExpo>
    ExpoOut,
    // Applies no transformation (default).
    #[default]
    Linear,
    /// <https://easings.net/#easeInQuad>
    QuadIn,
    /// <https://easings.net/#easeInOutQuad>
    QuadInOut,
    /// <https://easings.net/#easeOutQuad>
    QuadOut,
    /// <https://easings.net/#easeInQuart>
    QuartIn,
    /// <https://easings.net/#easeInOutQuart>
    QuartInOut,
    /// <https://easings.net/#easeOutQuart>
    QuartOut,
    /// <https://easings.net/#easeInQuint>
    QuintIn,
    /// <https://easings.net/#easeInOutQuint>
    QuintInOut,
    /// <https://easings.net/#easeOutQuint>
    QuintOut,
    // A linear easing that goes from 1.0 to 0.0.
    Reverse,
    // A linear easing that goes from 0.0 to 1.0 and back to 0.0. That might be used in combination with other easing functions.
    RoundTrip,
    /// <https://easings.net/#easeInSine>
    SineIn,
    /// <https://easings.net/#easeInOutSine>
    SineInOut,
    /// <https://easings.net/#easeOutSine>
    SineOut,
}

impl Easing {
    /// Call the easing function.
    pub(crate) fn call(&self, t: f32) -> f32 {
        match self {
            Easing::BackIn => back_in(t),
            Easing::BackInOuT => back_in_out(t),
            Easing::BackOut => back_out(t),
            Easing::BounceIn => bounce_in(t),
            Easing::BounceInOut => bounce_in_out(t),
            Easing::BounceOut => bounce_out(t),
            Easing::CircIn => circ_in(t),
            Easing::CircInOut => circ_in_out(t),
            Easing::CircOut => circ_out(t),
            Easing::CubicIn => cubic_in(t),
            Easing::CubicInOut => cubic_in_out(t),
            Easing::CubicOut => cubic_out(t),
            Easing::ElasticIn => elastic_in(t),
            Easing::ElasticInOut => elastic_in_out(t),
            Easing::ElasticOut => elastic_out(t),
            Easing::ExpoIn => expo_in(t),
            Easing::ExpoInOut => expo_in_out(t),
            Easing::ExpoOut => expo_out(t),
            Easing::Linear => t,
            Easing::QuadIn => quad_in(t),
            Easing::QuadInOut => quad_in_out(t),
            Easing::QuadOut => quad_out(t),
            Easing::QuartIn => quart_in(t),
            Easing::QuartInOut => quart_in_out(t),
            Easing::QuartOut => quart_out(t),
            Easing::QuintIn => quint_in(t),
            Easing::QuintInOut => quint_in_out(t),
            Easing::QuintOut => quint_out(t),
            Easing::Reverse => reverse(t),
            Easing::RoundTrip => roundtrip(t),
            Easing::SineIn => sine_in(t),
            Easing::SineInOut => sine_in_out(t),
            Easing::SineOut => sine_out(t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_easing_approx_equal(easing: Easing, input: f32, expected: f32) {
        let result = easing.call(input);
        assert!(
            (result - expected).abs() < 1e-6,
            "Expected {}, got {}",
            expected,
            result
        );
    }

    #[test]
    fn test_back_in() {
        assert_easing_approx_equal(Easing::BackIn, 0.0, 0.0);
        assert_easing_approx_equal(Easing::BackIn, 0.5, -0.0876975);
        assert_easing_approx_equal(Easing::BackIn, 1.0, 1.0);
    }

    #[test]
    fn test_back_in_out() {
        assert_easing_approx_equal(Easing::BackInOuT, 0.0, 0.0);
        assert_easing_approx_equal(Easing::BackInOuT, 0.2, -0.092556);
        assert_easing_approx_equal(Easing::BackInOuT, 0.5, 0.5);
        assert_easing_approx_equal(Easing::BackInOuT, 0.8, 1.0925556);
        assert_easing_approx_equal(Easing::BackInOuT, 1.0, 1.0);
    }

    #[test]
    fn test_back_out() {
        assert_easing_approx_equal(Easing::BackOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::BackOut, 0.5, 1.0876975);
        assert_easing_approx_equal(Easing::BackOut, 1.0, 1.0);
    }

    #[test]
    fn test_bounce_in() {
        assert_easing_approx_equal(Easing::BounceIn, 0.0, 0.0);
        assert_easing_approx_equal(Easing::BounceIn, 0.5, 0.234375);
        assert_easing_approx_equal(Easing::BounceIn, 1.0, 1.0);
    }

    #[test]
    fn test_bounce_in_out() {
        assert_easing_approx_equal(Easing::BounceInOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::BounceInOut, 0.2, 0.113750);
        assert_easing_approx_equal(Easing::BounceInOut, 0.5, 0.5);
        assert_easing_approx_equal(Easing::BounceInOut, 0.8, 0.88625);
        assert_easing_approx_equal(Easing::BounceInOut, 1.0, 1.0);
    }

    #[test]
    fn test_bounce_out() {
        assert_easing_approx_equal(Easing::BounceOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::BounceOut, 0.5, 0.765625);
        assert_easing_approx_equal(Easing::BounceOut, 1.0, 1.0);
    }

    #[test]
    fn test_circ_in() {
        assert_easing_approx_equal(Easing::CircIn, 0.0, 0.0);
        assert_easing_approx_equal(Easing::CircIn, 0.5, 0.133975);
        assert_easing_approx_equal(Easing::CircIn, 1.0, 1.0);
    }

    #[test]
    fn test_circ_in_out() {
        assert_easing_approx_equal(Easing::CircInOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::CircInOut, 0.2, 0.041742);
        assert_easing_approx_equal(Easing::CircInOut, 0.5, 0.5);
        assert_easing_approx_equal(Easing::CircInOut, 0.8, 0.958257);
        assert_easing_approx_equal(Easing::CircInOut, 1.0, 1.0);
    }

    #[test]
    fn test_circ_out() {
        assert_easing_approx_equal(Easing::CircOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::CircOut, 0.5, 0.866025);
        assert_easing_approx_equal(Easing::CircOut, 1.0, 1.0);
    }

    #[test]
    fn test_cubic_in() {
        assert_easing_approx_equal(Easing::CubicIn, 0.0, 0.0);
        assert_easing_approx_equal(Easing::CubicIn, 0.5, 0.125);
        assert_easing_approx_equal(Easing::CubicIn, 1.0, 1.0);
    }

    #[test]
    fn test_cubic_in_out() {
        assert_easing_approx_equal(Easing::CubicInOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::CubicInOut, 0.2, 0.032);
        assert_easing_approx_equal(Easing::CubicInOut, 0.5, 0.5);
        assert_easing_approx_equal(Easing::CubicInOut, 0.8, 0.968);
        assert_easing_approx_equal(Easing::CubicInOut, 1.0, 1.0);
    }

    #[test]
    fn test_cubic_out() {
        assert_easing_approx_equal(Easing::CubicOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::CubicOut, 0.5, 0.875);
        assert_easing_approx_equal(Easing::CubicOut, 1.0, 1.0);
    }

    #[test]
    fn test_elastic_in() {
        assert_easing_approx_equal(Easing::ElasticIn, 0.0, 0.0);
        assert_easing_approx_equal(Easing::ElasticIn, 0.5, -0.015625);
        assert_easing_approx_equal(Easing::ElasticIn, 1.0, 1.0);
    }

    #[test]
    fn test_elastic_in_out() {
        assert_easing_approx_equal(Easing::ElasticInOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::ElasticInOut, 0.2, -0.003906);
        assert_easing_approx_equal(Easing::ElasticInOut, 0.5, 0.5);
        assert_easing_approx_equal(Easing::ElasticInOut, 0.8, 1.0039063);
        assert_easing_approx_equal(Easing::ElasticInOut, 1.0, 1.0);
    }

    #[test]
    fn test_elastic_out() {
        assert_easing_approx_equal(Easing::ElasticOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::ElasticOut, 0.5, 1.015625);
        assert_easing_approx_equal(Easing::ElasticOut, 1.0, 1.0);
    }

    #[test]
    fn test_expo_in() {
        assert_easing_approx_equal(Easing::ExpoIn, 0.0, 0.0);
        assert_easing_approx_equal(Easing::ExpoIn, 0.5, 0.03125);
        assert_easing_approx_equal(Easing::ExpoIn, 1.0, 1.0);
    }

    #[test]
    fn test_expo_in_out() {
        assert_easing_approx_equal(Easing::ExpoInOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::ExpoInOut, 0.2, 0.007812);
        assert_easing_approx_equal(Easing::ExpoInOut, 0.5, 0.5);
        assert_easing_approx_equal(Easing::ExpoInOut, 0.8, 0.992187);
        assert_easing_approx_equal(Easing::ExpoInOut, 1.0, 1.0);
    }

    #[test]
    fn test_expo_out() {
        assert_easing_approx_equal(Easing::ExpoOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::ExpoOut, 0.5, 0.96875);
        assert_easing_approx_equal(Easing::ExpoOut, 1.0, 1.0);
    }

    #[test]
    fn test_linear() {
        assert_easing_approx_equal(Easing::Linear, 0.0, 0.0);
        assert_easing_approx_equal(Easing::Linear, 0.5, 0.5);
        assert_easing_approx_equal(Easing::Linear, 1.0, 1.0);
    }

    #[test]
    fn test_quad_in() {
        assert_easing_approx_equal(Easing::QuadIn, 0.0, 0.0);
        assert_easing_approx_equal(Easing::QuadIn, 0.5, 0.25);
        assert_easing_approx_equal(Easing::QuadIn, 1.0, 1.0);
    }

    #[test]
    fn test_quad_in_out() {
        assert_easing_approx_equal(Easing::QuadInOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::QuadInOut, 0.2, 0.08000);
        assert_easing_approx_equal(Easing::QuadInOut, 0.5, 0.5);
        assert_easing_approx_equal(Easing::QuadInOut, 0.8, 0.92);
        assert_easing_approx_equal(Easing::QuadInOut, 1.0, 1.0);
    }

    #[test]
    fn test_quad_out() {
        assert_easing_approx_equal(Easing::QuadOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::QuadOut, 0.5, 0.75);
        assert_easing_approx_equal(Easing::QuadOut, 1.0, 1.0);
    }

    #[test]
    fn test_quart_in() {
        assert_easing_approx_equal(Easing::QuartIn, 0.0, 0.0);
        assert_easing_approx_equal(Easing::QuartIn, 0.5, 0.0625);
        assert_easing_approx_equal(Easing::QuartIn, 1.0, 1.0);
    }

    #[test]
    fn test_quart_in_out() {
        assert_easing_approx_equal(Easing::QuartInOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::QuartInOut, 0.2, 0.0128);
        assert_easing_approx_equal(Easing::QuartInOut, 0.5, 0.5);
        assert_easing_approx_equal(Easing::QuartInOut, 0.8, 0.9872);
        assert_easing_approx_equal(Easing::QuartInOut, 1.0, 1.0);
    }

    #[test]
    fn test_quart_out() {
        assert_easing_approx_equal(Easing::QuartOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::QuartOut, 0.5, 0.9375);
        assert_easing_approx_equal(Easing::QuartOut, 1.0, 1.0);
    }

    #[test]
    fn test_quint_in() {
        assert_easing_approx_equal(Easing::QuintIn, 0.0, 0.0);
        assert_easing_approx_equal(Easing::QuintIn, 0.5, 0.0625);
        assert_easing_approx_equal(Easing::QuintIn, 1.0, 1.0);
    }

    #[test]
    fn test_quint_in_out() {
        assert_easing_approx_equal(Easing::QuintInOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::QuintInOut, 0.2, 0.00512);
        assert_easing_approx_equal(Easing::QuintInOut, 0.5, 0.5);
        assert_easing_approx_equal(Easing::QuintInOut, 0.8, 0.99488);
        assert_easing_approx_equal(Easing::QuintInOut, 1.0, 1.0);
    }

    #[test]
    fn test_quint_out() {
        assert_easing_approx_equal(Easing::QuintOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::QuintOut, 0.5, 0.96875);
        assert_easing_approx_equal(Easing::QuintOut, 1.0, 1.0);
    }

    #[test]
    fn test_reverse() {
        assert_eq!(Easing::Reverse.call(0.0), 1.0);
        assert_eq!(Easing::Reverse.call(0.5), 0.5);
        assert_eq!(Easing::Reverse.call(1.0), 0.0);
    }

    #[test]
    fn test_roundtrip() {
        assert_eq!(Easing::RoundTrip.call(0.0), 0.0);
        assert_eq!(Easing::RoundTrip.call(0.5), 1.0);
        assert_eq!(Easing::RoundTrip.call(1.0), 0.0);
    }

    #[test]
    fn test_sine_in() {
        assert_easing_approx_equal(Easing::SineIn, 0.0, 0.0);
        assert_easing_approx_equal(Easing::SineIn, 0.5, 0.292893);
        assert_easing_approx_equal(Easing::SineIn, 1.0, 1.0);
    }

    #[test]
    fn test_sine_in_out() {
        assert_easing_approx_equal(Easing::SineInOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::SineInOut, 0.2, 0.0954915);
        assert_easing_approx_equal(Easing::SineInOut, 0.5, 0.5);
        assert_easing_approx_equal(Easing::SineInOut, 0.8, 0.9045085);
        assert_easing_approx_equal(Easing::SineInOut, 1.0, 1.0);
    }

    #[test]
    fn test_sine_out() {
        assert_easing_approx_equal(Easing::SineOut, 0.0, 0.0);
        assert_easing_approx_equal(Easing::SineOut, 0.5, std::f32::consts::FRAC_1_SQRT_2);
        assert_easing_approx_equal(Easing::SineOut, 1.0, 1.0);
    }
}
