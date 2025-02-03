^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
Changelog for package visualization_msgs
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

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

1.12.6 (2018-05-03)
-------------------

1.12.5 (2016-09-30)
-------------------
* Fix spelling mistakes
* Contributors: trorornmn

1.12.4 (2016-02-22)
-------------------

1.12.3 (2015-04-20)
-------------------

1.12.2 (2015-03-21)
-------------------

1.12.1 (2015-03-17)
-------------------

1.12.0 (2014-12-29)
-------------------
* Enable DELETEALL=3 definition for marker action. Fixes `#39 <https://github.com/ros/common_msgs/issues/39>`_
* Contributors: Tully Foote

1.11.6 (2014-11-04)
-------------------

1.11.5 (2014-10-27)
-------------------

1.11.4 (2014-06-19)
-------------------
* Added comment to document new Rviz Marker action
* Contributors: Dave Coleman

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
* change comment to indicate 3D control modes can now be used with mouse.
* update email in package.xml

1.9.15 (2013-03-08)
-------------------

1.9.14 (2013-01-19)
-------------------

1.9.13 (2013-01-13)
-------------------

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
* Adds 3D control modes to interactive marker msgs
* updated cmake min version to 2.8.3, use cmake_parse_arguments instead of custom macro

1.9.5 (2012-09-28)
------------------
* fixed missing find genmsg

1.9.4 (2012-09-27 18:06)
------------------------

1.9.3 (2012-09-27 17:39)
------------------------
* cleanup
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
* fixed package dependency for another common message (`#3956 <https://github.com/ros/common_msgs/issues/3956>`_), removed unnecessary package name from another message
* fixed package dependencies for several common messages (fixed `#3956 <https://github.com/ros/common_msgs/issues/3956>`_)
* adding manifest exports
* removed depend, added catkin
* stripping depend and export tags from common_msgs manifests as msg dependencies are now declared in cmake and stack.yaml.  Also removed bag migration exports
* common_msgs: removing migration rules as all are over a year old
* bye bye vestigial MSG_DIRS
* visualization_msgs: catkin'd
* adios rosbuild2 in manifest.xml
* visualization_msgs: added 3D point of mouse event to InteractiveMarkerFeedback; fixed typo in comment in InteractiveMarkerControl.
* visualization_msgs: moved INIT function of InteractiveMarkerUpdate.msg into its own message: InteractiveMarkerInit.msg, in accordance with bug `#5021 <https://github.com/ros/common_msgs/issues/5021>`_
* visualization_msgs: updated InteractiveMarker, MenuEntry, and InteractiveMarkerFeedback messages and removed Menu message per API review decision about cleaning up menu specifications
* visualization_msgs: switched byte fields to uint8 per API review
* visualization_msgs: clarified comment per API review
* visualization_msgs: moved header to be first field of InteractiveMarkerFeedback per API review.
* visualization_msgs: comments clarified per API review.
* visualization_msgs: changed KEEP_ALIVE constant values in different messages to use the same value.
* - added mouse_down / mouse_up events
* changed layout of Menu messages
* updated documentation
* - added server_id, client_id; keep-alive and init updates; removed frame_locked option (now: timestamp=0)
* added header to i.m. feedback
* removed reference_frame again
* added reference_frame_id to i.m.
* updated docum., added descripion field to interactive marker, changed tool_tip to description in i.m. control
* updated feedback ducomentation
* PING->KEEP_ALIVE
* added PING feedback type
* added independent_marker_orientation to msg/InteractiveMarkerControl.msg
* updated interactive marker messages
* cleaned up the mess of commit `#36835 <https://github.com/ros/common_msgs/issues/36835>`_
* rosbuild2 updates
* renamed InteractiveMarkerArray to InteractiveMarkerUpdate
* updated msg/InteractiveMarkerFeedback.msg
* updated interactive marker spec
* updated documentation, namiing of fields for interactive **
* removed cpp interface from visualization_msgs again. too much work, too little outcome
* made a better cpp interface for interactive marker generation
* added view facing markers spec
* added more functions to include/visualization_msgs/interactive_marker_tools.h
* added initial version of interactive marker messages
* rosbuild2 taking shape
* removing old exports for msg/cpp and reving to 1.3.7 in preparation for release
* update rosbagmigration dependency (`#4510 <https://github.com/ros/common_msgs/issues/4510>`_)
* add visualization_msgs to common_msgs
