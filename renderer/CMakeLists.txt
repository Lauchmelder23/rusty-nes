project("rusty-nes")

add_subdirectory(vendor)

add_library(renderer STATIC
	context.c
)

target_include_directories(renderer PUBLIC
	${CMAKE_CURRENT_LIST_DIR}
)

target_link_libraries(renderer PRIVATE
	glad
)

install(TARGETS renderer glad DESTINATION .)