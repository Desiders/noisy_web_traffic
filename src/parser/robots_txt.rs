use texting_robots::Robot;

#[derive(Debug, thiserror::Error)]
#[error("Invalid robot rules for user agent {user_agent} ({raw}): {message}")]
pub struct InvalidRobotRules {
    user_agent: String,
    raw: String,
    message: String,
}

impl InvalidRobotRules {
    pub const fn new(user_agent: String, raw: String, message: String) -> Self {
        Self {
            user_agent,
            raw,
            message,
        }
    }
}

pub fn get_robot_rules(
    user_agent: &Option<impl AsRef<str>>,
    raw: &str,
) -> Result<Robot, InvalidRobotRules> {
    let user_agent = user_agent.as_ref().map_or("*", AsRef::as_ref);

    match Robot::new(user_agent, raw.as_bytes()) {
        Ok(robot) => Ok(robot),
        Err(error) => Err(InvalidRobotRules::new(
            user_agent.to_string(),
            raw.to_owned(),
            error.to_string(),
        )),
    }
}
