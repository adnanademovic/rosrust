^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
Changelog for package sensor_msgs
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

1.13.1 (2021-01-11)
-------------------
* Update package maintainers (`#168 <https://github.com/ros/common_msgs/issues/168>`_)
* Contributors: Michel Hidalgo

1.13.0 (2020-05-21)
-------------------
* Update BatteryState.msg (`#140 <https://github.com/ros/common_msgs/issues/140>`_)
* Use setuptools instead of distutils (`#159 <https://github.com/ros/common_msgs/issues/159>`_)
* Bump CMake version to avoid CMP0048 warning (`#158 <https://github.com/ros/common_msgs/issues/158>`_)
* Fix TabError: inconsistent use of tabs and spaces in indentation (`#155 <https://github.com/ros/common_msgs/issues/155>`_)
  * Fix TabError: inconsistent use of tabs and spaces in indentation
  Python 3 is much more strict for spacing.
* Contributors: Ramon Wijnands, Rein Appeldoorn, Shane Loretz

1.12.7 (2018-11-06)
-------------------
* Include sstream on header that needs i (`#131 <https://github.com/ros/common_msgs/issues/131>`_)
* included missing import for the read_points_list method (`#128 <https://github.com/ros/common_msgs/issues/128>`_)
  * included missing import for the read_points_list method
* Merge pull request `#127 <https://github.com/ros/common_msgs/issues/127>`_ from ros-1/fix-typos
* Merge pull request `#85 <https://github.com/ros/common_msgs/issues/85>`_ from ros/missing_test_target_dependency
  fix missing test target dependency
* Contributors: Dirk Thomas, Jasper, Kuang Fangjun, Tully Foote, chapulina

1.12.6 (2018-05-03)
-------------------
* Return default value to prevent missing return warning.
* Add function to convert PointCloud2 to namedtuples
  Add new function read_points_list that converts a PointCloud2 to a list of named tuples.
  It works on top of read_points, which generates lists containing the values.
  In consequence read_points_list is slower than read_points.
* Added equidistant distortion model const
* Added test_depend on rosunit in sensor_msgs
* fix catkin_lint warnings
* add mingration rule, copied from common_msgs-1.6
* Add missing include for atoi.
  Fixes `#97 <https://github.com/ros/common_msgs/issues/97>`_
* Contributors: 2scholz, Adam Allevato, Ivor Wanders, Kei Okada, Tully Foote, alexzzhu

1.12.5 (2016-09-30)
-------------------
* Deal with abstract image encodings
* Fix spelling mistakes
* Fix year
* Contributors: Jochen Sprickerhof, Kentaro Wada, trorornmn

1.12.4 (2016-02-22)
-------------------
* added type mapping and support for different types of points in point clouds
* remove boost dependency fixes `#81 <https://github.com/ros/common_msgs/issues/81>`_
* adding a BatteryState message
* fix iterator doc
* remove warning due to anonymous namespace
* Contributors: Sebastian Pütz, Tully Foote, Vincent Rabaud

1.12.3 (2015-04-20)
-------------------

1.12.2 (2015-03-21)
-------------------

1.12.1 (2015-03-17)
-------------------

1.12.0 (2014-12-29)
-------------------

1.11.6 (2014-11-04)
-------------------
* Fix compilation with Clang
* Contributors: jmtatsch

1.11.5 (2014-10-27)
-------------------
* add a test for the operator+ fix
  The behavior of that operator also had to be fixed to return a proper child class
* fix critical bug with operator+
* Contributors: Michael Ferguson, Vincent Rabaud

1.11.4 (2014-06-19)
-------------------
* Fix bug caused by use of va_arg in argument list.
* Contributors: Daniel Maturana

1.11.3 (2014-05-07)
-------------------
* clean up documentation of sensor_msgs so that wiki generated doxygen is readable
* Export architecture_independent flag in package.xml
* Contributors: Michael Ferguson, Scott K Logan

1.11.2 (2014-04-24)
-------------------

1.11.1 (2014-04-16)
-------------------
* fix missing include dirs for tests
* Contributors: Dirk Thomas

1.11.0 (2014-03-04)
-------------------
* add a PointCloud2 iterator and modifier
* Contributors: Tully Foote, Vincent Rabaud

1.10.6 (2014-02-27)
-------------------

1.10.5 (2014-02-25)
-------------------

1.10.4 (2014-02-18)
-------------------
* Fix roslib import for message module, remove roslib.load_manifest
* Contributors: Bence Magyar

1.10.3 (2014-01-07)
-------------------
* python 3 compatibility
* line wrap Imu.msg comments

1.10.2 (2013-08-19)
-------------------
* adding __init__.py `#11 <https://github.com/ros/common_msgs/issues/11>`_

1.10.1 (2013-08-16)
-------------------
* setup.py for `#11 <https://github.com/ros/common_msgs/issues/11>`_
* adding installation of point_cloud2.py to sensor_msgs fixes `#11 <https://github.com/ros/common_msgs/issues/11>`_

1.10.0 (2013-07-13)
-------------------
* adding MultiDOFJointState message

1.9.16 (2013-05-21)
-------------------
* update email in package.xml

1.9.15 (2013-03-08)
-------------------

1.9.14 (2013-01-19)
-------------------

1.9.13 (2013-01-13)
-------------------
* YUV422 actually has an 8 bit depth

1.9.12 (2013-01-02)
-------------------
* do not consider YUV422 to be a color anymore
* added missing license header

1.9.11 (2012-12-17)
-------------------
* modified dep type of catkin

1.9.10 (2012-12-13)
-------------------
* add missing downstream depend
* switched from langs to message_* packages

1.9.9 (2012-11-22)
------------------
* Added Low-Cost/Android Sensors reviewed messages
* Updated comments to reflect REP 117 for fixed-distance rangers
* Adding reviewed MultiEchoLaserScan message.

1.9.8 (2012-11-14)
------------------

1.9.7 (2012-10-30)
------------------
* fix catkin function order

1.9.6 (2012-10-18)
------------------
* updated cmake min version to 2.8.3, use cmake_parse_arguments instead of custom macro
* fix the bad number of channels for YUV422 UYVY

1.9.5 (2012-09-28)
------------------
* fixed missing find genmsg

1.9.4 (2012-09-27 18:06)
------------------------

1.9.3 (2012-09-27 17:39)
------------------------
* cleanup
* add precision about YUV422
* add YUV422 to some functions
* cleaned up package.xml files
* updated to latest catkin
* fixed dependencies and more
* updated to latest catkin: created package.xmls, updated CmakeLists.txt

1.9.2 (2012-09-05)
------------------
* updated pkg-config in manifest.xml
* updated catkin variables

1.9.1 (2012-09-04)
------------------
* use install destination variables, removed manual installation of manifests

1.9.0 (2012-08-29)
------------------
* update the docs

1.8.13 (2012-07-26 18:34:15 +0000)
----------------------------------
* made inline functions static inline
* fix ODR violation and missing headers
* moved c++ code from sensor_msgs to headers

1.8.8 (2012-06-12 22:36)
------------------------
* simplifying deps
* make find_package REQUIRED
* removed obsolete catkin tag from manifest files
* fixed package dependency for another common message (`#3956 <https://github.com/ros/common_msgs/issues/3956>`_), removed unnecessary package name from another message
* fixed package dependencies for several common messages (fixed `#3956 <https://github.com/ros/common_msgs/issues/3956>`_)
* clarify NavSatFix message comments
* normalize shared lib building, `#3838 <https://github.com/ros/common_msgs/issues/3838>`_
* adding TimeReference to build
* TimeReference decl was invalid
* adding point_cloud2 as reviewed at http://ros.org/wiki/sensor_msgs/Reviews/Python%20PointCloud2%20_API_Review
* TimeReference msg as reviewed #ros-pkg5355
* install headers
* adding manifest exports
* fix boost-finding stuff
* removed depend, added catkin
* adding roscpp_core dependencies
* stripping depend and export tags from common_msgs manifests as msg dependencies are now declared in cmake and stack.yaml.  Also removed bag migration exports
* install-related fixes
* common_msgs: removing migration rules as all are over a year old
* sensor_msgs: removing old octave support now that rosoct is gone
* bye bye vestigial MSG_DIRS
* sensor_msgs: getting rid of other build files
* adios rosbuild2 in manifest.xml
* catkin updates
* catkin_project
* Updated to work with new message generation macros
* adios debian/ hello stack.yaml.  (sketch/prototype/testing).
* More tweaking for standalone message generation
* Getting standalone message generation working... w/o munging rosbuild2
* more rosbuild2 hacking
* rosbuild2 tweaks
* missing dependencies
* sensor_msgs: Added YUV422 image encoding constant.
* adding in explicit ros/console.h include for ros macros now that ros::Message base class is gone
* adding JoyFeedback and JoyFeedbackArray
* updating manifest.xml
* adding Joy.msg
* Add image encodings for 16-bit Bayer, RGB, and BGR formats.
  Update isMono(), isAlpha(), isBayer(), etc.
* rosbuild2 taking shape
* sensor_msgs: Source-compatible corrections to fillImage signature.
* sensor_msgs: Functions for distinguishing categories of encodings. From cv_bridge redesign API review.
* applying patch to this method like josh did in r33966 in rviz
* sensor_msgs (rep0104): Migration rules for CameraInfo, RegionOfInterest.
* sensor_msgs (rep0104): Doc improvements for CameraInfo.
* sensor_msgs (rep0104): Cleaned up PointCloud2 msg docs. Restored original meaning of 'no invalid points' to is_dense (`#4446 <https://github.com/ros/common_msgs/issues/4446>`_).
* sensor_msgs (rep0104): Documented u,v channel semantics for PointCloud msg (`#4482 <https://github.com/ros/common_msgs/issues/4482>`_).
* sensor_msgs (rep0104): Added distortion model string constants.
* sensor_msgs (rep0104): Include guard for image_encodings.h.
* sensor_msgs (rep0104): Applied changes to CameraInfo and RegionOfInterest messages.
* Clarify frame of reference for NavSatFix position covariance.
* Add new satellite navigation messages approved by GPS API review.
* adding Range message as reviewed `#4488 <https://github.com/ros/common_msgs/issues/4488>`_
* adding missing file
* cleaner fix for point_cloud_conversion definitions for `#4451 <https://github.com/ros/common_msgs/issues/4451>`_
* inlining implementation in header for `#4451 <https://github.com/ros/common_msgs/issues/4451>`_
* sensor_msgs: Fixed URL in CameraInfo.msg and indicated how to mark an uncalibrated camera. `#4105 <https://github.com/ros/common_msgs/issues/4105>`_
* removing all the extra exports
* add units to message description
* bug fix in PC->PC2 conversion
* include guards for point_cloud_conversions.h `#4285 <https://github.com/ros/common_msgs/issues/4285>`_
* Added Ubuntu platform tags to manifest
* added PointCloud2<->PointCloud conversion routines.
* Updating link to camera calibration
* updating message as per review http://www.ros.org/wiki/sensor_msgs/Reviews/2010-03-01%20PointCloud2_API_Review
* sensor_msgs: Added size (number of elements for arrays) to PointField.
* pushing the new PointCloud structure in trunk
* Changed wording of angle convention for the LaserScan message. We are now specifying how angles are measured, not which way the laser spins.
* Remove use of deprecated rosbuild macros
* Added exporting of generated srv includes.
* Added call to gen_srv now that there is a service.
* Added the SetCameraInfo service.
* octave image parsing function now handles all possible image format types
* changing review status
* adding JointState documentation ticket:3006
* Typo in comments
* updated parsing routines for octave
* Adding 1 more rule for migration point clouds and bringing test_common_msgs back from future.
* Adding JointState migration rule.
* replace pr2_mechanism_msgs::JointStates by new non-pr2-specific sensor_msgs::JointState. Door test passes
* better documentation of the CameraInfo message
* updated url
* sensor_msgs: Added rule to migrate from old laser_scan/LaserScan.
* sensor_msgs: Added string constants for bayer encodings.
* clearing API reviews for they've been through a bunch of them recently.
* Removed the Timestamp message.
* Updating migration rules to better support the intermediate Image message that existed.
* comments for sensor_msgs
* Adding a CompressedImage migration rule.
* Fixing robot_msgs references
* Changing the ordering of fields within the new image message so that all meta information comes before the data block.
* Migration of RawStereo message.
* Migration rule for CameraInfo.
* First cut at migration rules for images.
* Moving stereo messages out of sensor_msgs to stereo/stereo_msgs
* Getting rid of PixelEncoding since it is encompassed in Image message instead.
* update to IMU message comments and defined semantics for covariance
* Changing naming of bag migration rules.
* Image message and CvBridge change
* moving FillImage.h to fill_image.h for Jeremy
* Adding image_encodings header/cpp, since genmsg_cpp doesn't actually support constant string values
* fixing spelling
* Message documentation
* Switching IMU to sensor_msgs/Imu related to `#2277 <https://github.com/ros/common_msgs/issues/2277>`_
* adding IMU msg
* Took out event_type field, as that would indeed make it more than a
  timestamp.
* adding OpenCV doc comment
* Rename rows,cols to height,width in Image message
* Adding more migration rule tests and fixing assorted rules.
* Added a timestamp message. (Will be used to track camera and perhaps some
  day hokuyo trigger times.)
* sensor_msgs: Updates to CameraInfo, added pixel encoding and ROI.
* New sensor_msgs::Image message
* PointCloud:
  * pts -> points
  * chan -> channels
  ChannelFloat32:
  * vals -> values
* sensor_msgs: Added explanation of reprojection matrix to StereoInfo.
* sensor_msgs: Cleaned up CompressedImage. Updated image_publisher. Blacklisted jpeg.
* merging in the changes to messages see ros-users email.  THis is about half the common_msgs API changes
* sensor_msgs: Comments to better describe CameraInfo and StereoInfo.
* Renamed CamInfo message to CameraInfo.
* sensor_msgs_processImage can now process empty images
* 
* update openrave and sensor_msgs octave scripts
* Image from image_msgs -> sensor_msgs `#1661 <https://github.com/ros/common_msgs/issues/1661>`_
* updating review status
* moving LaserScan from laser_scan package to sensor_msgs package `#1254 <https://github.com/ros/common_msgs/issues/1254>`_
* populating common_msgs
