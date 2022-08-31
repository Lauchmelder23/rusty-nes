project("rusty-nes")

add_subdirectory(vendor)

add_library(renderer STATIC
	context.c
)

target_link_libraries(renderer PRIVATE
	glad
)

install(TARGETS renderer glad DESTINATION .)