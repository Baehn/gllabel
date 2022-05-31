# Modify these INCLUDES and LIBS paths to match your system configuration.
# The current values are for macOS, with the glfw, glew, glm, and freetype2
# packages all installed via Homebrew.

# OpenGL
GL_INCLUDES=
GL_LIBS=-lglut -lGLU -lGL

# GLFW: For creating the demo window
GLFW_INCLUDES=-I/usr/include/GLFW
GLFW_LIBS=-lglfw

# GLEW: OpenGL extension loader
GLEW_INCLUDES=-I/usr/include/GL
GLEW_LIBS=-lGLEW

# GLM: Matrix math
GLM_INCLUDES=-I/usr/include

# FreeType2: For reading TrueType font files
# FT2_INCLUDES=-I/usr/include/freetype2
# FT2_LIBS=-lfreetype


CC=g++
CPPFLAGS=-Wall -Wextra -g -std=c++14 -Iinclude ${GL_INCLUDES} ${GLFW_INCLUDES} ${GLEW_INCLUDES} ${GLM_INCLUDES} 
LDLIBS=${GL_LIBS} ${GLFW_LIBS} ${GLEW_LIBS}  -lflib -L ./target/debug/ -Wl,-rpath,./target/debug/

run: demo
	./demo

demo: demo.cpp lib/gllabel.cpp lib/types.cpp lib/util.cpp lib/vgrid.cpp 
