function build(b) { // b is a Builder Context. All of the Basalt api is exposed through this
	const clang = b.toolchains.clang(b);

    const raylib = b.dependency("raylib"); // dependency finds an external dependency, currently
                                           // only over pkg-config
    const foo = b.dependency("foo::foo");
    const bar = b.dependency("bar::bar");
                                           
    const hello = b.executable("hello", { // Create an executable target
        sources: b.glob("src/**/*.c"),
        dependencies: [raylib, foo, bar], // Link with Raylib, foo and bar.
                                          // Dependencies works with both external dependencies
                                          // and local targets
        toolchain: clang,
    	includeDirectories: ["src/"],
    	define: []
    });

    return [hello]; // All targets that should be built must be returned
                              // Order doesnt matter, dependency tree will be resolved.
}
