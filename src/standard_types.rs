use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

// Message Types

/// Invalid message type. This message type should not be used.
pub const MSG_TYPE_INVALID: i32            = 0;
/// Ping message type (Service). Request and reply payload is `Ping`.
pub const MSG_TYPE_PING: i32               = 1;
/// Get version message type (Service). Request payload is empty; reply payload is `GetVersion`.
pub const MSG_TYPE_GET_VERSION: i32        = 2;
/// Joint data message type (Topic). Payload is `JointMessage`.
pub const MSG_TYPE_JOINT: i32              = 10;
/// Joint trajectory point message type (Service). Request payload is `JointTrajPt`. Reply payload can be `Ping` or
/// empty.
pub const MSG_TYPE_JOINT_TRAJ_PT: i32      = 11;
/// Joint trajectory message type (Service). Request payload is `JointTraj` or `JointTrajFixed`. Reply payload can be
/// `Ping` or empty.
pub const MSG_TYPE_JOINT_TRAJ: i32         = 12;
/// Robot status message type (Topic). Payload is `RobotStatus`.
pub const MSG_TYPE_STATUS: i32             = 13;
/// Joint trajectory point message type (Service). Request payload is `JointTrajPtFull`. Reply payload can be `Ping` or
/// empty.
pub const MSG_TYPE_JOINT_TRAJ_PT_FULL: i32 = 14;
/// Joint pos/vel/accel feedback message type (Topic). Payload is `JointFeedback`.
pub const MSG_TYPE_JOINT_FEEDBACK: i32     = 15;
/// Read input message type. Payload is undocumented in the original source (ros-industrial/industrial_code).
pub const MSG_TYPE_READ_INPUT: i32         = 20;
/// Write output message type. Payload is undocumented in the original source (ros-industrial/industrial_code).
pub const MSG_TYPE_WRITE_OUTPUT: i32       = 21;

// Vendor specific message type ranges.
// Only define the beginning enum value, specific enum values should be defined locally, within in the range reserved
// here. Each vendor can reserve up 100 types

/// Start of SWRI specific message types. 100 message types are reserved for SWRI specific messages.
pub const MSG_TYPE_BEGIN_SWRI: i32         = 1000;
/// Start of Universal Robots specific message types. 100 message types are reserved for UR specific messages.
pub const MSG_TYPE_BEGIN_UR: i32           = 1100;
/// Start of Adept specific message types. 100 message types are reserved for Adept specific messages.
pub const MSG_TYPE_BEGIN_ADEPT: i32        = 1200;
/// Start of ABB specific message types. 100 message types are reserved for ABB specific messages.
pub const MSG_TYPE_BEGIN_ABB: i32          = 1300;
/// Start of Fanuc specific message types. 100 message types are reserved for Fanuc specific messages.
pub const MSG_TYPE_BEGIN_FANUC: i32        = 1400;
/// Start of Motoman specific message types. 100 message types are reserved for Motoman specific messages.
pub const MSG_TYPE_BEGIN_MOTOMAN: i32      = 2000;

/// Sequence number.
///
/// Zero and positive values indicate the 0-based index of the element in the current sequence. Negative values have
/// special meaning and are defined in the `SEQ_*` constants below.
pub use i32 as SeqNum;
/// Downloading drivers only: signals start
pub const SEQ_START_TRAJECTORY_DOWNLOAD: i32 = -1;
/// Signal start of trajectory
pub const SEQ_START_TRAJECTORY_STREAMING: i32 = -2;
/// Downloading drivers only: signals end
pub const SEQ_END_TRAJECTORY: i32 = -3;
/// Driver must abort any currently executing motion
pub const SEQ_STOP_TRAJECTORY: i32 = -4;

/// Bitmask indicating which fields in a message contain a valid payload.
///
/// Bit positions are defined in the `VALID_*` constants below. For example, if the message contains position and
/// acceleration, the ValidFields value should be set to `VALID_POSITION | VALID_ACCELERATION`.
pub use i32 as ValidFields;
/// The 'time' field contains valid data
pub const VALID_TIME: i32 = 0x1;
/// The 'positions' field contains valid data
pub const VALID_POSITION: i32 = 0x2;
/// The 'velocities' field contains valid data
pub const VALID_VELOCITY: i32 = 0x4;
/// The 'accelerations' field contains valid data
pub const VALID_ACCELERATION: i32 = 0x8;

/// Tri-state value.
///
/// Tri-state values are used for fields that can be in one of three states: on, off, or unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum TriState {
    /// Unknown state.
    Unknown = -1,
    /// Also encodes False, Disabled, or Low
    Off = 0,
    /// Also encodes True, Enabled, or High
    On = 1,
}

/// Ping message payload.
///
/// This message may be used by clients to test communication with the server.
///
/// Server implementations should respond to incoming PING messages with minimal delay.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ping {
    /// Ping data payload. The REP speecifies that this data is be ignored by both client and server, but also specifies
    /// that it must be initialized to all zeroes.
    pub dummy_data: [i32; 10],
}

/// Get version message payload.
///
/// Allows clients to determine the specific version of a server implementation running on the remote system. This
/// version number may be specific to the server, and thus cannot be used to compare different server implementations.
///
/// *Note: Server implementations may return alphanumeric version info in any of the major, minor or patch fields, but
/// this may result in rendering artefacts on the client side. The generic clients in industrial_robot_client will
/// always interpret these fields as signed integers.*
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GetVersion {
    /// The major version number of the server.
    pub major: i32,
    /// The minor version number of the server.
    pub minor: i32,
    /// The patch version number of the server.
    pub patch: i32,
}

/// Maximum number of joints in the JointData message.
pub const MAX_NUM_JOINTS: usize = 10;
/// Joint data payload.
///
///
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct JointData {
    /// The joint positions. For simplicity and cross platform compliance, this is implemented as a fixed size array.
    ///
    /// Elements of joint_data that are not used must be initialised to zero (0) by the sender.
    pub joints: [f32; MAX_NUM_JOINTS],
}

/// Joint data message payload.
///
/// This message was part of the first set of messages supported by the generic clients that servers could use to report
/// joint states. There is no support for joint velocity, acceleration or effort, nor a group identifier or index. The
/// message size is fixed and the maximum number of joints supported is ten (10).
///
/// Early server implementations also accepted this message for enqueuing trajectory points. This usage has been
/// deprecated (and support removed from industrial_robot_client) and it is an error for clients to try to use
/// JOINT_POSITION for this purpose. Driver authors must use JOINT_TRAJ_PT and JOINT_TRAJ_PT_FULL messages instead.
///
/// *Note that use of this message for encoding joint states is also deprecated (because of the lack of support for
/// motion groups, joint velocity or accelerations mentioned earlier in this section), and new server implementations
/// are recommended to use JOINT_FEEDBACK to encode joint states.*
///
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct JointMessage {
    /// The sequence number of the joint message.
    ///
    /// The sequence field is not used when reporting joint state and shall be set to zero by server implementations.
    pub sequence: SeqNum,
    /// The joint positions.
    pub joint_data: JointData,
}

/// Joint trajectory.
///
/// A joint trajectory includes an array of JointTrajPt data types. The intention for this struct is to be loaded into a
/// single message for communication over a simple connection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JointTraj {
    /// The array of trajectory points.
    ///
    /// *Note*: The standard specifies that this field is preceded by a size field indicating its size. This is
    /// implicitly encoded by the simple_message crate, and does not need to be explicitly specified in this struct.
    ///
    /// The specification suggests that this array should nonetheless be exactly 10 elements long, with unused elements
    /// set to zero. This implementation does not support this; if the user pads this vector to a length of 10 with zero
    /// data for any unused elements, the size will be encoded as 10, rather than indicating the number of elements that
    /// are used. If the user desires the fixed-size array behavior, they should use the JointTrajFixed struct instead.
    pub points: Vec<JointTrajPt>,
}

/// Maximum number of trajectory points in the JointtTrajFixed message.
pub const MAX_TRAJ_SIZE: usize = 10;
/// Joint trajectory wwith fixed-size payload.
///
/// A joint trajectory includes an array of JointTrajPt data types. The intention for this struct is to be loaded into a
/// single message for communication over a simple connection.
///
/// For simplicity and cross platform compliance, this is implemented as a fixed size array. The size of the trajectory
/// cannot exceed the max size of the array.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct JointTrajFixed {
    /// The number of trajectory points in the array that are used. This cannot exceed MAX_TRAJ_SIZE.
    pub size: i32,
    /// The array of trajectory points.
    ///
    /// Any unused elements must be initialised to zero (0) by the sender.
    pub points: [JointTrajPt; MAX_TRAJ_SIZE],
}

/// Joint trajectory point data.
///
/// The point data serves as a waypoint along a trajectory and is meant to mirror the JointTrajectoryPoint message.
///
/// This point differs from the ROS trajectory point in the following ways:
///
///  - The joint velocity is specified in an industrial robot standard way (as a single value).
///  - The duration is how long the move should take. A big assumption is that a sequence of points is continuously
///    executed. This is generally true of a ROS trajectory but not required.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct JointTrajPt {
    /// The sequence number of the trajectory point.
    pub sequence: SeqNum,
    /// The joint positions.
    pub joints: JointData,
    /// The velocity of the trajectory point, specified in a form specific to the robot controller.
    pub velocity: f32,
    /// The duration the move should take, specified in a form specific to the robot controller.
    pub duration: f32,
}

/// Joint trajectory point data.
///
/// The point data serves as a waypoint along a trajectory and is meant to mirror the JointTrajectoryPoint message.
///
/// Similar to JOINT_TRAJ_PT, this message may be used by clients to enqueue trajectory points for execution by the
/// server. In addition to position, JOIN_TRAJ_PT_FULL allows clients to encode velocity and acceleration, as well as to
/// target messages at specific motion groups.
///
/// This message is intended for use with servers that provide sufficient control over both motion planning and
/// trajectory execution.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct JointTrajPtFull {
    /// The numeric identifier of the corresponding motion group on the controller. This field uses zero-based counting.
    ///
    /// In cases where motion groups are not identified by numeric ids on the controller, drivers shall implement an
    /// appropriate mapping (ie: alphabetical sorting of group names, etc).
    pub robot_id: i32,
    /// The sequence number of the trajectory point.
    pub sequence: SeqNum,
    /// Bitmask indicating which fields in the message contain a valid payload.
    pub valid_fields: ValidFields,
    /// The time at which the trajectory point should be executed, specified in a form specific to the robot controller.
    pub time: f32,
    /// The joint positions, specified in a form specific to the robot controller.
    pub positions: JointData,
    /// The joint velocities, specified in a form specific to the robot controller.
    pub velocities: JointData,
    /// The joint accelerations, specified in a form specific to the robot controller.
    pub accelerations: JointData,
}

/// Joint feedback message data.
///
/// The analogue of JOINT_TRAJ_PT_FULL, but for robot joint state reporting.
///
/// This data represents the current state of each robot joint, including position/velocity/acceleration.
/// The specific interpretation of this data (actual vs. commanded, timestamp, etc.) is up to the robot-controller
/// implementation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct JointFeedback {
    /// The numeric identifier of the corresponding motion group on the controller. This field uses zero-based counting.
    ///
    /// In cases where motion groups are not identified by numeric ids on the controller, drivers shall implement an
    /// appropriate mapping (ie: alphabetical sorting of group names, etc).
    pub robot_id: i32,
    /// Bitmask indicating which fields in the message contain a valid payload.
    pub valid_fields: ValidFields,
    /// The time that the feedback data was sampled, specified in a form specific to the robot controller.
    pub time: f32,
    /// The joint positions, specified in a form specific to the robot controller.
    pub positions: JointData,
    /// The joint velocities, specified in a form specific to the robot controller.
    pub velocities: JointData,
    /// The joint accelerations, specified in a form specific to the robot controller.
    pub accelerations: JointData,
}

/// Robot status data.
///
/// The STATUS message may be used by servers to inform clients of the general status of the controller, including
/// whether the controller is currently in an error state, whether the emergency stop is active, whether any attached
/// robot is executing a motion and what the current operating mode of the controller is.
///
/// This version of STATUS can only encode an aggregate state, so drivers for controllers with multiple motion groups
/// will need to determine how to merge group state into an aggregate controller state.
///
/// The robot status data is meant to mirror the industrial_msgs/RobotStatus message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RobotStatus {
    /// Whether the drives are powered on.
    pub drives_powered: TriState,
    /// Whether the emergency stop is active.
    pub e_stopped: TriState,
    /// The error code of the controller (specific to the controller).
    pub error_code: i32,
    /// Whether the controller is in an error state.
    pub in_error: TriState,
    /// Whether the controller is executing a motion.
    pub in_motion: TriState,
    /// The current operating mode of the controller (specific to the controller).
    pub mode: i32,
    /// Whether the controller is able to execute motions.
    pub motion_possible: TriState,
}
