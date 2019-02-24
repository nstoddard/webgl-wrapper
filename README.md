This is a WIP and isn't complete or efficient yet.

This library is somewhat similar to [glium](https://github.com/glium/glium); the main differences are that this library supports WebGL through `web-sys` while AFAIK glium only supports WebGL through stdweb, this library only implements a subset of OpenGL functionality (though more functionality can be added as needed), and some parts of the API (such as meshes) are higher-level.

Once [gfx-rs](https://github.com/gfx-rs/gfx) works with WebGL, this library will most likely be deprecated.

Current features:

* Programs, meshes, 2D textures, and basic support for framebuffers and renderbuffers
* State caching to reduce the number of redundant OpenGL calls

Features not yet implemented:

* Deletion of OpenGL objects which are no longer used
* An easier way to implement the `Vertex` and `Uniforms` traits
* More usage examples
* More types of textures
