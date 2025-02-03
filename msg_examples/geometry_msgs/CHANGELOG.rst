^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
Changelog for package geometry_msgs
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

1.13.1 (2021-01-11)
-------------------
* Update package maintainers (`#168 <https://github.com/ros/common_msgs/issues/168>`_)
* Contributors: Michel Hidalgo

1.13.0 (2020-05-21)
-------------------
* Bump CMake version to avoid CMP0048 warning (`#158 <https://github.com/ros/common_msgs/issues/158>`_)
* Contributors: Shane Loretz

1.12.7 (2018-11-06)
-------------------
* Add deprecation comment about Pose2D (`#130 <https://github.com/ros/common_msgs/issues/130>`_)
  It started as unused and still isn't recommended.
  Followup to `#129 <https://github.com/ros/common_msgs/issues/129>`_
* Contributors: Tully Foote

1.12.6 (2018-05-03)
-------------------

1.12.5 (2016-09-30)
-------------------
* Fix spelling mistakes
* Contributors: trorornmn

1.12.4 (2016-02-22)
-------------------
* clarify the definition of a Vector3
* Contributors: Vincent Rabaud

1.12.3 (2015-04-20)
-------------------
* geometry_msgs/InertiaStamped uses geometry_msgs/Inertia.
* Contributors: Gayane Kazhoyan, Georg Bartels

1.12.2 (2015-03-21)
-------------------
* Add Accel, AccelStamped, AccelWithCovariance, AccelWithCovarianceStamped message definitions
* Add Inertia and InertiaStamped messages
* Contributors: Jonathan Bohren, Paul Bovbel

1.12.1 (2015-03-17)
-------------------
* updating outdated urls. fixes `#52 <https://github.com/ros/common_msgs/issues/52>`_.
* Contributors: Tully Foote

1.12.0 (2014-12-29)
-------------------

1.11.6 (2014-11-04)
-------------------

1.11.5 (2014-10-27)
-------------------

1.11.4 (2014-06-19)
-------------------

1.11.3 (2014-05-07)
-------------------
* Export architecture_independent flag in package.xml
* Contributors: Scott K Logan

1.11.2 (2014-04-24)
-------------------

1.11.1 (2014-04-16)
-------------------

1.11.0 (2014-03-04)
-------------------

1.10.6 (2014-02-27)
-------------------

1.10.5 (2014-02-25)
-------------------

1.10.4 (2014-02-18)
-------------------

1.10.3 (2014-01-07)
-------------------

1.10.2 (2013-08-19)
-------------------

1.10.1 (2013-08-16)
-------------------

1.10.0 (2013-07-13)
-------------------

1.9.16 (2013-05-21)
-------------------
* update email in package.xml

1.9.15 (2013-03-08)
-------------------

1.9.14 (2013-01-19)
-------------------

1.9.13 (2013-01-13)
-------------------
* fix spelling in comments (fix `#3194 <https://github.com/ros/common_msgs/issues/3194>`_)

1.9.12 (2013-01-02)
-------------------

1.9.11 (2012-12-17)
-------------------
* modified dep type of catkin

1.9.10 (2012-12-13)
-------------------
* add missing downstream depend
* switched from langs to message_* packages

1.9.9 (2012-11-22)
------------------

1.9.8 (2012-11-14)
------------------

1.9.7 (2012-10-30)
------------------
* fix catkin function order

1.9.6 (2012-10-18)
------------------
* updated cmake min version to 2.8.3, use cmake_parse_arguments instead of custom macro

1.9.5 (2012-09-28)
------------------

1.9.4 (2012-09-27 18:06)
------------------------

1.9.3 (2012-09-27 17:39)
------------------------
* cleanup
* cleaned up package.xml files
* updated to latest catkin
* fixed dependencies and more
* updated to latest catkin: created package.xmls, updated CmakeLists.txt

1.9.2 (2012-09-05)
------------------
* updated pkg-config in manifest.xml

1.9.1 (2012-09-04)
------------------
* use install destination variables, removed manual installation of manifests

1.9.0 (2012-08-29)
------------------

1.8.13 (2012-07-26 18:34:15 +0000)
----------------------------------

1.8.8 (2012-06-12 22:36)
------------------------
* removed obsolete catkin tag from manifest files
* removed unnecessary package name from some messages
* adding manifest exports
* removed depend, added catkin
* stripping depend and export tags from common_msgs manifests as msg dependencies are now declared in cmake and stack.yaml.  Also removed bag migration exports
* install-related fixes
* common_msgs: removing migration rules as all are over a year old
* bye bye vestigial MSG_DIRS
* geometry_msgs: getting rid of other build files
* updated to new catkin_export_python macro
* adios rosbuild2 in manifest.xml
* catkin updates
* catkin_project
* Updated to work with new message generation macros
* adios debian/ hello stack.yaml.  (sketch/prototype/testing).
* Getting standalone message generation working... w/o munging rosbuild2
* rosbuild2 tweaks
* initial updating for new light message generation and wgbuild
* missing dependencies
* updating bagmigration exports
* rosbuild2 taking shape
* workaround bug #ros3018 until ros-1.2.3 comes out
* removing all the extra exports
* Added Ubuntu platform tags to manifest
* Remove use of deprecated rosbuild macros
* link to tf package as per doc review
* doc reviewed status
* wrapping manifest nicely
* updated url and description
* full migration rules
* switching TransformStamped logic to follow that of all other frame_ids where the frame_id is the operating frame and there is now a child_frame_id which defines the target frame.  And the parent frame is gone.  This is only changing the message.  The API change will come later.
* making covariance follow same convention as Pose
* rotation representation was specified the wrong way in the message comment
* Adding a stamped version of polygon
* Adding comment to Polygon message
* Adding migration rule from ParticleCloud to PoseArray
* clearing API reviews for they've been through a bunch of them recently.
* comments on all msgs except Polygon
* removing PoseWithRates as it's deprecated.
* Changing naming of bag migration rules.
* Modifying migration rules for Odometry and WrenchStamped change of field names.
* Adding actual migration rules for all of the tested common_msgs migrations.
* undo of `#2270 <https://github.com/ros/common_msgs/issues/2270>`_, (.data for stamped). reverts r21133
* Adding migration rules to get migration tests to pass.
* switching from PosewithRatesStamped to Odometry `#2277 <https://github.com/ros/common_msgs/issues/2277>`_
* Fixing some of the migration rules associated with unrolling of the .data change.
* PoseWithCovarianceStamped::data -> PoseWithCovarianceStamped::pose
* Reverse r21134, PointStamped::point->PointStamped::data
* reverse QuaternionStamped::quaternion -> QuaternionStamped::data change
* undoing r21137, keeping Vector3Stamped as was, but keeping in fix to door_handle_detector 'using' bug
* Adding more migration rule tests and fixing assorted rules.
* reverting r2118. Redoing `#2275 <https://github.com/ros/common_msgs/issues/2275>`_ `#2274 <https://github.com/ros/common_msgs/issues/2274>`_ to not go to 'data' standard
* `#2271 <https://github.com/ros/common_msgs/issues/2271>`_ Vector3Stamped uses new standarization
* PointStamped::point -> PointStamped::data (`#2276 <https://github.com/ros/common_msgs/issues/2276>`_)
* new Stamped format `#2270 <https://github.com/ros/common_msgs/issues/2270>`_
* Changing migration rule for Twist to go to TwistStamped.
* QuaternionStamped::quaternion -> QuaternionStamped::data (`#2278 <https://github.com/ros/common_msgs/issues/2278>`_)
* `#2274 <https://github.com/ros/common_msgs/issues/2274>`_ `#2275 <https://github.com/ros/common_msgs/issues/2275>`_ updated to header/data
* PoseWithCovariance->PoseWithCovarianceStamped
  PoseWithCovarianceStamped::pose_with_covariance -> PoseWithCovarianceStamped::data
* First half of the change from deprecated_msgs::RobotBase2DOdom to nav_msgs::Odometry, I think all the c++ compiles, can't speak for functionality yet, also... the python has yet to be run... this may break some things
* Moved robot_msgs/Polygon3D to geometry_msgs/Polygon for ticket `#1310 <https://github.com/ros/common_msgs/issues/1310>`_
* moving PoseArray into geometry_msgs `#1907 <https://github.com/ros/common_msgs/issues/1907>`_
* removing header for this is a type for composing and doesn't stand on it's own to be transformed etc.
* adding TwistWithCovariance `#2251 <https://github.com/ros/common_msgs/issues/2251>`_
* creating Wrench and WrenchStamped in geometry_msgs `#1935 <https://github.com/ros/common_msgs/issues/1935>`_
* adding unused Pose2D message as per API review `#2249 <https://github.com/ros/common_msgs/issues/2249>`_
* geometry_msgs: Documented that covariance uses fixed axis not euler angles.
* merging in the changes to messages see ros-users email.  THis is about half the common_msgs API changes
