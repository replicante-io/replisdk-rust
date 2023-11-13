//! RepliCore Authentication and Authorisation entity, action and resource models.
//!
//! First of all:
//!
//! - Authentication: answers "who is asking for access?" (it is about identity).
//! - Authorisation: answers "can they do what they are asking to do?" (it is about access).
//!
//! ## Entities
//!
//! Entities represent who is accessing the system (be it a human or another system).
//!
//! The aim of different [`Entity`] types is to better handler different use cases.
//! For example:
//!
//! - A hunan user may have multiple roles to represent the different "hats" they were in a day.
//! - A service account really has only one purpose and therefore only has one role.
//!
//! ### Impersonation
//!
//! Impersonation is a technique that allows an entity (user or system) to perform a request
//! acting as a different entity.
//!
//! Impersonation can be very useful to:
//!
//! - Minimise day-to-day access while enabling occasional escalation to greater privileges.
//! - Test and troubleshoot access issues.
//!
//! ## Actions
//!
//! Different resources support different actions.
//! To handle this the case actions are represented as string with the format `{scope}:{action}`.
//!
//! The downside of the above flexibility is that an exhaustive list of actions can only
//! be derived by combining all actions for all scopes wherever they are defined.
//!
//! The use of the [`Action`] type enables both type safety in code
//! as well as a quick search pattern to find where action lists are defined.
//!
//! ## Resources
//!
//! [`Resource`]s are things (data, actions, etc ...) in the RepliCore system that
//! [`Entity`]s can perform [`Action`]s on.
//!
//! All [`Resource`]s have a `kind` that describe what they are as well as a `resource_id`.
//!
//! Additionally `Resource`s can have arbitrary metadata useful when determining access.
//! For example all namespaced resources have their namespace attached to the attributes
//! with the [`RESOURCE_NAMESPACE`] key.
use std::collections::BTreeMap;

use serde::de::Deserialize;
use serde::de::Deserializer;
use serde::de::Error;
use serde::de::Unexpected;
use serde::de::Visitor;
use serde::Serialize;
use serde::Serializer;

/// For namespaced resources this is the metadata key where their namespace is set.
pub const RESOURCE_NAMESPACE: &str = "namespace";

/// Wrapper around [`String`]s to ensure correct action semantics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Action {
    /// String the action is stored in.
    inner: String,
}

impl Action {
    /// Define a scoped action.
    pub fn define(scope: &str, action: &str) -> Action {
        let inner = format!("{}:{}", scope, action);
        Action { inner }
    }
}

impl From<Action> for String {
    fn from(value: Action) -> Self {
        value.inner
    }
}

impl AsRef<str> for Action {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl std::borrow::Borrow<str> for Action {
    fn borrow(&self) -> &str {
        &self.inner
    }
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// Process `&str` or `String` from serde to deserialize an [`Action`].
        struct VisitAction;
        impl<'de> Visitor<'de> for VisitAction {
            type Value = Action;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string in the {scope}:{action} format")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if check_value(v) {
                    let inner = v.to_string();
                    Ok(Action { inner })
                } else {
                    Err(Error::invalid_value(Unexpected::Str(v), &self))
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if check_value(&v) {
                    Ok(Action { inner: v })
                } else {
                    Err(Error::invalid_value(Unexpected::Str(&v), &self))
                }
            }
        }

        /// Check a `&str` or `String` for validity without allocations.
        fn check_value(v: &str) -> bool {
            match v.split_once(':') {
                None => false,
                Some((scope, _)) if scope.is_empty() => false,
                Some((_, action)) if action.is_empty() => false,
                _ => true,
            }
        }

        // Trigger deserialization of an Action.
        deserializer.deserialize_string(VisitAction)
    }
}

impl Serialize for Action {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.inner)
    }
}

/// Authentication and authorisation information carried by RepliCore's Context.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, serde::Deserialize)]
pub struct AuthContext {
    /// The action being performed.
    pub action: Action,

    /// The entity (user or system) requesting the action.
    pub entity: Entity,

    /// The entity to impersonate when processing the request.
    pub impersonate: Option<ImpersonateEntity>,

    /// The resource the action is to be performed on.
    pub resource: Resource,
}

/// An entity is someone (a user) or something (a service) performing an action.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, serde::Deserialize)]
#[serde(tag = "kind")]
pub enum Entity {
    /// No identity information is available for the request.
    #[serde(rename = "anonymous")]
    Anonymous,

    /// The request is made by a service account.
    #[serde(rename = "service")]
    Service(EntityService),

    /// The request originates from the Control Plane itself.
    ///
    /// Actions initiated by the system are always allowed on all resources.
    /// This entity and its processing through authorisation backends is performed mainly
    /// for auditing and troubleshooting reasons.
    #[serde(rename = "system")]
    System(EntitySystem),

    /// The request is made by a user account.
    #[serde(rename = "user")]
    User(EntityUser),
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anonymous => write!(f, "anonymous"),
            Self::Service(service) => write!(f, "service:{}", service.service_id),
            Self::System(system) => write!(f, "system:{}", system.component),
            Self::User(user) => write!(f, "user:{}", user.user_id),
        }
    }
}

/// Information specific to service account entities.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, serde::Deserialize)]
pub struct EntityService {
    /// ID of the role the service account is authorised as.
    pub role: String,

    /// Identifier of the service performing the request (mainly for audits and logs).
    pub service_id: String,
}

/// Information specific to the Control Plane entities.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, serde::Deserialize)]
pub struct EntitySystem {
    /// The Control Plane component initiating the action.
    pub component: String,
}

/// Information specific for user account entities.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, serde::Deserialize)]
pub struct EntityUser {
    /// Custom metadata attached to a user identity.
    pub metadata: BTreeMap<String, String>,

    /// All roles a user is authorized to act as.
    pub roles: Vec<String>,

    /// Identifier of a user session the request is part of.
    pub session_id: String,

    /// Identifier of the user performing the request (mainly for audits and logs).
    pub user_id: String,
}

/// Subset of [`Entity`]s that can be impersonated by other entities.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, serde::Deserialize)]
#[serde(tag = "kind")]
pub enum ImpersonateEntity {
    /// The request is made as a service account.
    #[serde(rename = "service")]
    Service(EntityService),

    /// The request is made as a user account.
    #[serde(rename = "user")]
    User(EntityUser),
}

impl std::fmt::Display for ImpersonateEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Service(service) => write!(f, "service:{}", service.service_id),
            Self::User(user) => write!(f, "user:{}", user.user_id),
        }
    }
}

/// Control Plane resource the entity wants to perform an action on.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, serde::Deserialize)]
pub struct Resource {
    /// Family of the target resource.
    pub kind: String,

    /// Custom metadata attached to the target resource.
    pub metadata: BTreeMap<String, String>,

    /// Identifier of the target resource.
    pub resource_id: String,
}

#[cfg(test)]
mod tests {
    use serde_test::assert_de_tokens_error;
    use serde_test::assert_tokens;
    use serde_test::Token;

    use super::Action;

    const FAIL_CASES: [&str; 4] = ["", "test", "test:", ":test"];
    const SUCCESS_CASES: [(&str, &str, &str); 2] = [
        ("test", "action", "test:action"),
        ("test", "action:with:colon", "test:action:with:colon"),
    ];

    #[test]
    fn fail_cases_str() {
        for case in FAIL_CASES {
            let error = format!(
                "invalid value: string \"{}\", expected a string in the {{scope}}:{{action}} format",
                case,
            );
            assert_de_tokens_error::<Action>(&[Token::Str(case)], &error);
        }
    }

    #[test]
    fn fail_cases_string() {
        for case in FAIL_CASES {
            let error = format!(
                "invalid value: string \"{}\", expected a string in the {{scope}}:{{action}} format",
                case,
            );
            assert_de_tokens_error::<Action>(&[Token::String(case.into())], &error);
        }
    }

    #[test]
    fn success_cases_str() {
        for (scope, action, token) in SUCCESS_CASES {
            let action = Action::define(scope, action);
            assert_tokens(&action, &[Token::Str(token)]);
        }
    }

    #[test]
    fn success_cases_string() {
        for (scope, action, token) in SUCCESS_CASES {
            let action = Action::define(scope, action);
            assert_tokens(&action, &[Token::String(token)]);
        }
    }
}
