# Basalt
Basalt is a basic build system that uses QuickJS

To get started, create a file called `basalt.config.js` in the root of your project.
You can start with something like:
```js
function build(b) { // b is a Builder Context. All of the Basalt api is exposed through this
	const clang = b.toolchain({ // A toolchain is compilers, linkers and other tools
		cc: b.findProgram("clang"), // Find program looks for a program in path
		cxx: b.findProgram("clang++"),
		ld: b.findProgramOr(["mold", "ld.lld", "ld"]), // findProgramOr works like a list of priorities
		                                               // linker is passed by invoking clang with -fuse-ld
		ar: b.findProgram("ar")
	});

    const raylib = b.dependency("raylib"); // dependency finds an external dependency, currently
                                           // only over pkg-config

	const foo = b.library("foo", { // Create a library target
		sources: b.glob("foo/src/**/*.c"), // recursively find all c files in foo/src/
		dependencies: [raylib], // Link with Raylib, and also inherit its includes, etc
		toolchain: clang, // Use the clang toolchain from earlier
		isStatic: true, // Compile to a static library (.a)
		includeDirectories: ["foo/src/"], // Include directories for compilation
    	define: [] // Macros to define
	});

	const bar = b.library("bar", {
		sources: b.glob("bar/src/**/*.c"),
		dependencies: [raylib],
		toolchain: clang,
		isStatic: true, // Produce a .so file
		includeDirectories: ["bar/src/"],
    	define: []
	});
		
    const hello = b.executable("hello", { // Create an executable target
        sources: b.glob("src/**/*.c"),
        dependencies: [raylib, foo, bar], // Link with Raylib, foo and bar.
                                          // Dependencies works with both external dependencies
                                          // and local targets
        toolchain: clang,
    	includeDirectories: ["src/"],
    	define: []
    });

    return [hello, foo, bar]; // All targets that should be built must be returned
                              // Order doesnt matter, dependency tree will be resolved.
}

```

Now, you can run `basalt` to build your project.
To run a target after building, use `basalt run {target}`
Additionally, add `basalt -g {make,ninja} ...` to specifically use make or ninja
