#[derive(Clone, Copy)]
pub enum AnimationState {
    AnimatingIn,
    AnimatingOut,
    Idle,
    Offscreen,
}

pub struct AnimationController {
    pub animation: Animation,
    pub state: AnimationState,
    pub travel_distance: f64,
    on_complete: Option<Box<dyn FnOnce() -> Option<AnimationState> + Send>>,
}

impl AnimationController {
    pub fn new(travel_distance: f64, duration: f64) -> Self {
        Self {
            animation: Animation::new(duration, bouncy_easing),
            state: AnimationState::Offscreen,
            travel_distance,
            on_complete: None,
        }
    }

    pub fn start(
        &mut self,
        state: AnimationState,
        current_time: f64,
        on_complete: Option<Box<dyn FnOnce() -> Option<AnimationState> + Send>>,
    ) {
        self.state = state;
        self.animation.start(current_time);
        self.on_complete = on_complete;
    }

    pub fn update(&mut self, current_time: f64) -> f64 {
        let result = match self.state {
            AnimationState::AnimatingIn => {
                (1.0 - self.animation.sample(current_time).unwrap_or(0.0)) * self.travel_distance
            }
            AnimationState::AnimatingOut => {
                self.animation.sample(current_time).unwrap_or(0.0) * self.travel_distance
            }
            AnimationState::Offscreen => self.travel_distance,
            AnimationState::Idle => 0.0,
        };

        if self.animation.is_complete(current_time) {
            self.state = match self.state {
                AnimationState::AnimatingIn => AnimationState::Idle,
                AnimationState::AnimatingOut => AnimationState::Offscreen,
                _ => self.state,
            };
            if let Some(callback) = self.on_complete.take() {
                if let Some(next_animation) = callback() {
                    self.start(next_animation, current_time, None);
                }
            }
        }

        result
    }

    pub fn is_idle(&self) -> bool {
        matches!(self.state, AnimationState::Idle)
    }
}

#[derive(Clone)]
pub struct Animation {
    start_time: Option<f64>,
    duration: f64,
    easing: fn(f64) -> f64,
}

impl Animation {
    pub fn new(duration: f64, easing: fn(f64) -> f64) -> Self {
        Self {
            start_time: None,
            duration,
            easing,
        }
    }

    pub fn start(&mut self, current_time: f64) {
        self.start_time = Some(current_time);
    }

    pub fn progress(&self, current_time: f64) -> Option<f64> {
        self.start_time.map(|start| {
            let elapsed = current_time - start;
            (elapsed / self.duration).clamp(0.0, 1.0)
        })
    }

    pub fn is_complete(&self, current_time: f64) -> bool {
        self.progress(current_time).map_or(false, |p| p >= 1.0)
    }

    pub fn sample(&self, current_time: f64) -> Option<f64> {
        self.progress(current_time)
            .map(|progress| (self.easing)(progress))
    }
}

pub fn bouncy_easing(progress: f64) -> f64 {
    let (x1, y1) = (0.68, -0.6);
    let (x2, y2) = (0.32, 1.2);

    let t = solve_cubic_bezier(progress, x1, x2);
    cubic_bezier(t, y1, y2)
}

fn solve_cubic_bezier(progress: f64, x1: f64, x2: f64) -> f64 {
    let mut t = progress;
    for _ in 0..10 {
        let x = cubic_bezier(t, x1, x2);
        let dx = cubic_bezier_derivative(t, x1, x2);
        if dx.abs() < 1e-6 {
            break;
        }
        t -= (x - progress) / dx;
        t = t.clamp(0.0, 1.0);
    }
    t
}

fn cubic_bezier(t: f64, p1: f64, p2: f64) -> f64 {
    (1.0 - t).powi(3) * 0.0
        + 3.0 * (1.0 - t).powi(2) * t * p1
        + 3.0 * (1.0 - t) * t.powi(2) * p2
        + t.powi(3) * 1.0
}

fn cubic_bezier_derivative(t: f64, p1: f64, p2: f64) -> f64 {
    3.0 * (1.0 - t).powi(2) * p1 + 6.0 * (1.0 - t) * t * p2 + 3.0 * t.powi(2) * 1.0
}
