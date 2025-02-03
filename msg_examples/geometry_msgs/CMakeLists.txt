cmake_minimum_required(VERSION 3.0.2)
project(geometry_msgs)

find_package(catkin REQUIRED COMPONENTS message_generation std_msgs)

add_message_files(
  DIRECTORY msg
  FILES
  Accel.msg
  AccelStamped.msg
  AccelWithCovariance.msg
  AccelWithCovarianceStamped.msg
  Inertia.msg
  InertiaStamped.msg
  Point.msg
  Point32.msg
  PointStamped.msg
  Polygon.msg
  PolygonStamped.msg
  Pose2D.msg
  Pose.msg
  PoseArray.msg
  PoseStamped.msg
  PoseWithCovariance.msg
  PoseWithCovarianceStamped.msg
  Quaternion.msg
  QuaternionStamped.msg
  Transform.msg
  TransformStamped.msg
  Twist.msg
  TwistStamped.msg
  TwistWithCovariance.msg
  TwistWithCovarianceStamped.msg
  Vector3.msg
  Vector3Stamped.msg
  Wrench.msg
  WrenchStamped.msg)

generate_messages(DEPENDENCIES std_msgs)

catkin_package(CATKIN_DEPENDS message_runtime std_msgs)
