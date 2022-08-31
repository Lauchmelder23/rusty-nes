#include <assert.h>
#include <stdio.h>
#include <glad/glad.h>

static void* _glfw = NULL;
static void*(*_gl_loader)(void*, const char*) = NULL;

void* load_gl_proc(const char* name)
{
	assert(_glfw && _gl_loader);

	return _gl_loader(_glfw, name);
}

int init_opengl(void* glfw, void*(*gl_loader)(void*, const char*))
{
	_glfw = glfw;
	_gl_loader = gl_loader;

	if (!gladLoadGLLoader(load_gl_proc)) {
		return 1;
	}

	return 0;
}

void clear()
{
	assert(_glfw && _gl_loader);

	glClearColor(0.0f, 0.0f, 0.0f, 0.0f);
	glClear(GL_COLOR_BUFFER_BIT);
}