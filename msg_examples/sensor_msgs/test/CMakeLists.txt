include_directories(${catkin_INCLUDE_DIRS})
catkin_add_gtest(${PROJECT_NAME}_test main.cpp)
catkin_add_gtest(${PROJECT_NAME}_test_image_encodings test_image_encodings.cpp)
if(TARGET sensor_msgs_test)
  add_dependencies(${PROJECT_NAME}_test ${sensor_msgs_EXPORTED_TARGETS})
endif()

