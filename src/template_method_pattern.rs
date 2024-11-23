use rand_distr::StandardNormal;
use rand::Rng;

pub trait MCSimulation {
    /// Generates a vector of standard normal random numbers.
    fn generate_random_numbers(&self, num: usize) -> Vec<f64> {
        let mut rng = rand::thread_rng();
        (0..num).map(|_| rng.sample(StandardNormal)).collect()
    }

    /// Template method that runs the simulation.
    fn simulation(&self) -> Vec<f64> {
        let random_numbers = self.generate_random_numbers(self.get_number_of_steps());
        self.generate_path(&random_numbers)
    }

    /// Abstract method to generate the path based on random numbers.
    fn generate_path(&self, random_numbers: &[f64]) -> Vec<f64>;

    /// Returns the number of steps in the simulation.
    fn get_number_of_steps(&self) -> usize;
}
pub trait StochasticProcess {
    fn drift(&self, dt: f64) -> f64;
    fn diffusion(&self, dt: f64) -> f64;
}

pub struct GeometricBrownianMotion {
    pub initial_value: f64,
    pub risk_free_rate: f64,
    pub volatility: f64,
    pub time_steps: usize,
    pub maturity: f64,
}
impl StochasticProcess for GeometricBrownianMotion {
    fn drift(&self, _dt: f64) -> f64 {
        (self.risk_free_rate - 0.5 * self.volatility.powi(2))* _dt
    }
    fn diffusion(&self, _dt: f64) -> f64 {
        self.volatility * _dt.sqrt()
    }
}
impl MCSimulation for GeometricBrownianMotion {
    fn get_number_of_steps(&self) -> usize {
        self.time_steps
    }
    fn generate_path(&self, random_numbers: &[f64]) -> Vec<f64> {
        let dt = self.maturity / self.time_steps as f64;
        let mut s = self.initial_value;
        let mut path = Vec::with_capacity(self.time_steps + 1);
        path.push(s);
        for &dw in random_numbers {
            let drift = self.drift(dt);
            let diffusion = self.diffusion(dt) * dw ;
            s = s * (drift + diffusion).exp();
            path.push(s);
        }
        path
    }
}

pub struct Vasicek {
    pub initial_value: f64,
    pub risk_free_rate: f64,
    pub mean_reversion: f64,
    pub volatility: f64,
    pub time_steps: usize,
    pub maturity: f64,
}
impl StochasticProcess for Vasicek {
    fn drift(&self, dt: f64) -> f64 {
        self.mean_reversion * (self.risk_free_rate - dt)
    }
    fn diffusion(&self, _dt: f64) -> f64 {
        self.volatility * _dt.sqrt()
    }
}
impl MCSimulation for Vasicek {
    fn get_number_of_steps(&self) -> usize {
        self.time_steps
    }
    fn generate_path(&self, random_numbers: &[f64]) -> Vec<f64> {
        let dt = self.maturity / self.time_steps as f64;
        let mut r = self.initial_value;
        let mut path = Vec::with_capacity(self.time_steps + 1);
        path.push(r);
        for &dw in random_numbers {
            let drift = self.drift(dt);
            let diffusion = self.diffusion(dt) * dw ;
            r = r + drift + diffusion;
            path.push(r);
        }
        path
    }
}

fn main() {
    let gbm = GeometricBrownianMotion {
        initial_value: 100.0,
        risk_free_rate: 0.05,
        volatility: 0.2,
        time_steps: 1000,
        maturity: 1.0,
    };

    let path = gbm.simulation();
    println!("Generated GBM path: {:?}", path);
    let vasicek = Vasicek {
        initial_value: 0.05,
        risk_free_rate: 0.05,
        mean_reversion: 0.01,
        volatility: 0.2,
        time_steps: 1000,
        maturity: 1.0,
    };
    let path = vasicek.simulation();
    println!("Generated Vasicek path: {:?}", path);

}
