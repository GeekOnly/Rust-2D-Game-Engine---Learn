//! Easing functions for UI animations
//!
//! Provides a comprehensive set of easing functions for smooth animations.
//! All easing functions take a normalized time value (0.0 to 1.0) and return
//! a normalized progress value.

use std::f32::consts::PI;

use super::EasingFunction;

/// Evaluate an easing function at time t (0.0 to 1.0)
pub fn evaluate(easing: &EasingFunction, t: f32) -> f32 {
    // Clamp t to [0, 1] range
    let t = t.clamp(0.0, 1.0);
    
    match easing {
        EasingFunction::Linear => linear(t),
        
        EasingFunction::EaseInQuad => ease_in_quad(t),
        EasingFunction::EaseOutQuad => ease_out_quad(t),
        EasingFunction::EaseInOutQuad => ease_in_out_quad(t),
        
        EasingFunction::EaseInCubic => ease_in_cubic(t),
        EasingFunction::EaseOutCubic => ease_out_cubic(t),
        EasingFunction::EaseInOutCubic => ease_in_out_cubic(t),
        
        EasingFunction::EaseInQuart => ease_in_quart(t),
        EasingFunction::EaseOutQuart => ease_out_quart(t),
        EasingFunction::EaseInOutQuart => ease_in_out_quart(t),
        
        EasingFunction::EaseInQuint => ease_in_quint(t),
        EasingFunction::EaseOutQuint => ease_out_quint(t),
        EasingFunction::EaseInOutQuint => ease_in_out_quint(t),
        
        EasingFunction::EaseInSine => ease_in_sine(t),
        EasingFunction::EaseOutSine => ease_out_sine(t),
        EasingFunction::EaseInOutSine => ease_in_out_sine(t),
        
        EasingFunction::EaseInExpo => ease_in_expo(t),
        EasingFunction::EaseOutExpo => ease_out_expo(t),
        EasingFunction::EaseInOutExpo => ease_in_out_expo(t),
        
        EasingFunction::EaseInCirc => ease_in_circ(t),
        EasingFunction::EaseOutCirc => ease_out_circ(t),
        EasingFunction::EaseInOutCirc => ease_in_out_circ(t),
        
        EasingFunction::EaseInElastic => ease_in_elastic(t),
        EasingFunction::EaseOutElastic => ease_out_elastic(t),
        EasingFunction::EaseInOutElastic => ease_in_out_elastic(t),
        
        EasingFunction::EaseInBack => ease_in_back(t),
        EasingFunction::EaseOutBack => ease_out_back(t),
        EasingFunction::EaseInOutBack => ease_in_out_back(t),
        
        EasingFunction::EaseInBounce => ease_in_bounce(t),
        EasingFunction::EaseOutBounce => ease_out_bounce(t),
        EasingFunction::EaseInOutBounce => ease_in_out_bounce(t),
    }
}

// Linear
fn linear(t: f32) -> f32 {
    t
}

// Quadratic
fn ease_in_quad(t: f32) -> f32 {
    t * t
}

fn ease_out_quad(t: f32) -> f32 {
    t * (2.0 - t)
}

fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        -1.0 + (4.0 - 2.0 * t) * t
    }
}

// Cubic
fn ease_in_cubic(t: f32) -> f32 {
    t * t * t
}

fn ease_out_cubic(t: f32) -> f32 {
    let t = t - 1.0;
    t * t * t + 1.0
}

fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let t = 2.0 * t - 2.0;
        1.0 + t * t * t / 2.0
    }
}

// Quartic
fn ease_in_quart(t: f32) -> f32 {
    t * t * t * t
}

fn ease_out_quart(t: f32) -> f32 {
    let t = t - 1.0;
    1.0 - t * t * t * t
}

fn ease_in_out_quart(t: f32) -> f32 {
    if t < 0.5 {
        8.0 * t * t * t * t
    } else {
        let t = t - 1.0;
        1.0 - 8.0 * t * t * t * t
    }
}

// Quintic
fn ease_in_quint(t: f32) -> f32 {
    t * t * t * t * t
}

fn ease_out_quint(t: f32) -> f32 {
    let t = t - 1.0;
    1.0 + t * t * t * t * t
}

fn ease_in_out_quint(t: f32) -> f32 {
    if t < 0.5 {
        16.0 * t * t * t * t * t
    } else {
        let t = 2.0 * t - 2.0;
        1.0 + t * t * t * t * t / 2.0
    }
}

// Sine
fn ease_in_sine(t: f32) -> f32 {
    1.0 - (t * PI / 2.0).cos()
}

fn ease_out_sine(t: f32) -> f32 {
    (t * PI / 2.0).sin()
}

fn ease_in_out_sine(t: f32) -> f32 {
    -(((PI * t).cos()) - 1.0) / 2.0
}

// Exponential
fn ease_in_expo(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else {
        2.0_f32.powf(10.0 * t - 10.0)
    }
}

fn ease_out_expo(t: f32) -> f32 {
    if t == 1.0 {
        1.0
    } else {
        1.0 - 2.0_f32.powf(-10.0 * t)
    }
}

fn ease_in_out_expo(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else if t < 0.5 {
        2.0_f32.powf(20.0 * t - 10.0) / 2.0
    } else {
        (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0
    }
}

// Circular
fn ease_in_circ(t: f32) -> f32 {
    1.0 - (1.0 - t * t).sqrt()
}

fn ease_out_circ(t: f32) -> f32 {
    let t = t - 1.0;
    (1.0 - t * t).sqrt()
}

fn ease_in_out_circ(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
    } else {
        let t = -2.0 * t + 2.0;
        ((1.0 - t.powi(2)).sqrt() + 1.0) / 2.0
    }
}

// Elastic
fn ease_in_elastic(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        let c4 = (2.0 * PI) / 3.0;
        -2.0_f32.powf(10.0 * t - 10.0) * ((t * 10.0 - 10.75) * c4).sin()
    }
}

fn ease_out_elastic(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        let c4 = (2.0 * PI) / 3.0;
        2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
    }
}

fn ease_in_out_elastic(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        let c5 = (2.0 * PI) / 4.5;
        if t < 0.5 {
            -(2.0_f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
        } else {
            (2.0_f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0 + 1.0
        }
    }
}

// Back
fn ease_in_back(t: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    c3 * t * t * t - c1 * t * t
}

fn ease_out_back(t: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    let t = t - 1.0;
    1.0 + c3 * t * t * t + c1 * t * t
}

fn ease_in_out_back(t: f32) -> f32 {
    let c1 = 1.70158;
    let c2 = c1 * 1.525;
    
    if t < 0.5 {
        let t = 2.0 * t;
        (t * t * ((c2 + 1.0) * t - c2)) / 2.0
    } else {
        let t = 2.0 * t - 2.0;
        (t * t * ((c2 + 1.0) * t + c2) + 2.0) / 2.0
    }
}

// Bounce
fn ease_out_bounce(t: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;
    
    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t = t - 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        let t = t - 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / d1;
        n1 * t * t + 0.984375
    }
}

fn ease_in_bounce(t: f32) -> f32 {
    1.0 - ease_out_bounce(1.0 - t)
}

fn ease_in_out_bounce(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - ease_out_bounce(1.0 - 2.0 * t)) / 2.0
    } else {
        (1.0 + ease_out_bounce(2.0 * t - 1.0)) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        assert_eq!(linear(0.0), 0.0);
        assert_eq!(linear(0.5), 0.5);
        assert_eq!(linear(1.0), 1.0);
    }

    #[test]
    fn test_easing_bounds() {
        let easings = vec![
            EasingFunction::Linear,
            EasingFunction::EaseInQuad,
            EasingFunction::EaseOutQuad,
            EasingFunction::EaseInOutQuad,
            EasingFunction::EaseInCubic,
            EasingFunction::EaseOutCubic,
            EasingFunction::EaseInOutCubic,
        ];

        for easing in easings {
            let start = evaluate(&easing, 0.0);
            let end = evaluate(&easing, 1.0);
            
            assert!((start - 0.0).abs() < 0.001, "Easing {:?} should start at 0", easing);
            assert!((end - 1.0).abs() < 0.001, "Easing {:?} should end at 1", easing);
        }
    }

    #[test]
    fn test_easing_monotonic() {
        // Most easing functions should be monotonically increasing
        let easings = vec![
            EasingFunction::Linear,
            EasingFunction::EaseInQuad,
            EasingFunction::EaseOutQuad,
            EasingFunction::EaseInOutQuad,
        ];

        for easing in easings {
            let mut prev = 0.0;
            for i in 0..=10 {
                let t = i as f32 / 10.0;
                let val = evaluate(&easing, t);
                assert!(val >= prev, "Easing {:?} should be monotonic at t={}", easing, t);
                prev = val;
            }
        }
    }
}
