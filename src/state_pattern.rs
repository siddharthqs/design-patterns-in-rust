use std::collections::HashMap;
use std::{fmt, thread};
use std::sync::mpsc;
use std::sync::mpsc::Sender;

#[derive(Debug,Clone)]
pub enum TradingEngineCommand {
    ExecuteTrade,
    NoTrade,
    StopEngine,
}
pub trait RiskState: fmt::Debug {
    fn check_var(&self, context: &RiskManager) -> Option<Box<dyn RiskState>>;
    fn enter_state(&self, context: &RiskManager);
    fn exit_state(&self, context: &RiskManager);
    fn send_command(&self, context: &RiskManager);
}

#[derive(Debug)]
pub struct RiskManager {
    state: Box<dyn RiskState>,
    pub var_limit: f64,
    pub warning_level: f64,
    pub current_var: f64,
    pub positions: HashMap<String, f64>, // Position ID -> VaR contribution
    trading_engine_sender: Sender<TradingEngineCommand>,
}

impl RiskManager {
    pub fn new(var_limit: f64, warning_level: f64, trading_engine_sender: Sender<TradingEngineCommand>) -> Self {
        let mut manager = RiskManager {
            state: Box::new(NormalOperationState{cmd: TradingEngineCommand::ExecuteTrade}),
            var_limit,
            warning_level,
            current_var: 0.0,
            positions: HashMap::new(),
            trading_engine_sender,
        };
        manager
    }

    pub fn update_var(&mut self) {
        self.current_var = self.positions.values().sum();
    }

    pub fn add_position(&mut self, position_id: &str, var_contribution: f64) {
        self.positions.insert(position_id.to_string(), var_contribution);
        self.update_var();
        self.check_state();
        self.send_command();
    }

    pub fn remove_position(&mut self, position_id: &str) {
        self.positions.remove(position_id);
        self.update_var();
        self.check_state();
        self.send_command();
    }
    pub fn send_command(&self) {
        self.state.send_command(&self);
    }
    pub fn check_state(&mut self) {
        match self.state.check_var(self){
            Some(state) => self.change_state(state),
            None => (),
        }
    }

    pub fn change_state(&mut self, new_state: Box<dyn RiskState>) {
        self.state.exit_state(self);
        self.state = new_state;
        self.state.enter_state(self);
    }

    pub fn should_shutdown(&self) -> bool {
        self.current_var >= self.var_limit * 1.2
    }
}

#[derive(Debug)]
struct NormalOperationState{
    cmd: TradingEngineCommand
}

impl RiskState for NormalOperationState {
    fn check_var(&self, context: &RiskManager) -> Option<Box<dyn RiskState>> {
        if context.current_var >= context.warning_level && context.current_var < context.var_limit {
            Some(Box::new(WarningLevelState{cmd: TradingEngineCommand::ExecuteTrade}))
        } else if context.current_var >= context.var_limit {
            Some(Box::new(LimitBreachState{cmd: TradingEngineCommand::NoTrade}))
        } else {
            None
        }
    }

    fn enter_state(&self, _context: &RiskManager) {
        println!("Entering Normal Operation State");
        // Reset limitations
    }

    fn exit_state(&self, _context: &RiskManager) {
        println!("Exiting Normal Operation State");
    }
    fn send_command(&self, context: &RiskManager) {
        context.trading_engine_sender.send(self.cmd.clone());
    }
}

#[derive(Debug)]
struct WarningLevelState{
    cmd: TradingEngineCommand
}

impl RiskState for WarningLevelState {
    fn check_var(&self, context: &RiskManager) -> Option<Box<dyn RiskState>> {
        if context.current_var < context.warning_level {
            //context.change_state(Box::new(NormalOperationState));
            Some(Box::new(NormalOperationState{cmd: TradingEngineCommand::ExecuteTrade}))
        } else if context.current_var >= context.var_limit {
            //context.change_state(Box::new(LimitBreachState));
            Some(Box::new(LimitBreachState{cmd: TradingEngineCommand::NoTrade}))
        } else {
            None
        }
    }

    fn enter_state(&self, _context: &RiskManager) {
        println!("Entering Warning Level State");
        send_email_notification("Warning Level reached. Increased monitoring and trading limitations are in effect.");
    }

    fn exit_state(&self, _context: &RiskManager) {
        println!("Exiting Warning Level State");
    }
    fn send_command(&self, context: &RiskManager) {
        context.trading_engine_sender.send(TradingEngineCommand::ExecuteTrade);
    }
}

#[derive(Debug)]
struct LimitBreachState{
    cmd: TradingEngineCommand
}

impl RiskState for LimitBreachState {
    fn check_var(&self, context: &RiskManager) -> Option<Box<dyn RiskState>> {
        if context.current_var < context.var_limit && context.current_var >= context.warning_level {
            //context.change_state(Box::new(WarningLevelState));
            Some(Box::new(WarningLevelState{cmd: TradingEngineCommand::ExecuteTrade}))
        } else if context.current_var < context.warning_level {
            //context.change_state(Box::new(NormalOperationState));
            Some(Box::new(NormalOperationState{cmd: TradingEngineCommand::ExecuteTrade}))
        } else if context.should_shutdown() {
            //context.change_state(Box::new(ShutdownState));
            Some(Box::new(ShutdownState{cmd: TradingEngineCommand::StopEngine}))
        } else {
            None
        }
    }

    fn enter_state(&self, _context: &RiskManager) {
        println!("Entering Limit Breach State");
        send_email_notification("Limit Breach! New trades are blocked. Positions may be closed.");
    }

    fn exit_state(&self, _context: &RiskManager) {
        println!("Exiting Limit Breach State");
    }
    fn send_command(&self, context: &RiskManager) {
        context.trading_engine_sender.send(self.cmd.clone());
    }
}

#[derive(Debug)]
struct ShutdownState{
    cmd: TradingEngineCommand
}

impl RiskState for ShutdownState {
    fn check_var(&self, _context: &RiskManager) -> Option<Box<dyn RiskState>> {
        // Remain in ShutdownState
        None
    }

    fn enter_state(&self, _context: &RiskManager) {
        println!("Entering Shutdown State");
        send_email_notification("Shutdown initiated due to extreme risk levels.");
        self.send_command(_context);
    }

    fn exit_state(&self, _context: &RiskManager) {
        println!("Exiting Shutdown State");
    }
    fn send_command(&self, context: &RiskManager) {
        context.trading_engine_sender.send(self.cmd.clone());
    }
}

fn send_email_notification(message: &str) {
    println!("Sending email notification: {}", message);
}

pub struct TradingEngine;
impl TradingEngine {
    pub fn start(receiver: mpsc::Receiver<TradingEngineCommand>) {
        thread::spawn(move || {
            println!("Trading engine started.");
            loop{
                let cmd = receiver.recv().unwrap();
                match cmd {
                    TradingEngineCommand::ExecuteTrade => {
                        println!("Executing trade");
                    }
                    TradingEngineCommand::StopEngine => {
                        println!("Stopping trading engine.");
                        // Perform cleanup if necessary
                        break;
                    }
                    TradingEngineCommand::NoTrade => {
                        println!("No trade to execute.");
                    }
                    // Handle other commands if necessary
                }
            }
            println!("Trading engine stopped.");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn test_risk_manager() {
        let var_limit = 100.0;
        let warning_level = 80.0;
        let (trading_engine_sender, trading_engine_receiver) = mpsc::channel();
        TradingEngine::start(trading_engine_receiver);

        let mut risk_manager = RiskManager::new(var_limit, warning_level, trading_engine_sender);
        std::thread::sleep(std::time::Duration::from_secs(1));
        risk_manager.add_position("Position1", 30.0);
        assert_eq!(risk_manager.current_var, 30.0);

        risk_manager.add_position("Position2", 40.0);
        assert_eq!(risk_manager.current_var, 70.0);

        risk_manager.add_position("Position3", 20.0);
        assert_eq!(risk_manager.current_var, 90.0);
        std::thread::sleep(std::time::Duration::from_secs(2));
        risk_manager.add_position("Position4", 15.0);
        assert_eq!(risk_manager.current_var, 105.0);

        risk_manager.add_position("Position5", 35.0);
        assert_eq!(risk_manager.current_var, 140.0);
        // Simulate the passage of time and check if shutdown is needed
        risk_manager.check_state();
        std::thread::sleep(std::time::Duration::from_secs(2));
        risk_manager.check_state();
    }
}