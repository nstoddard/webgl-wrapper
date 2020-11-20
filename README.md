**[This library is deprecated, use `gl-wrapper` instead](https://github.com/nstoddard/gl-wrapper)**

A stateless wrapper around WebGL 2, to make it easier to use and more type-safe.

This library is somewhat similar to [glium](https://github.com/glium/glium); the main differences are that this library supports WebGL through `web-sys` while AFAIK glium only supports WebGL through stdweb, this library only implements a subset of OpenGL functionality (though more functionality can be added as needed), and some parts of the API (such as meshes) are higher-level.

Current features:

* Programs, meshes, 2D textures, and basic support for framebuffers and renderbuffers
* State caching to reduce the number of redundant OpenGL calls
* Instancing

Features not yet implemented:

* An easier way to implement the `Vertex` and `Uniforms` traits
* More usage examples
* More types of textures
