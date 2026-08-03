function build(b) { // b is a Builder Context. All of the Basalt api is exposed through this
	const clang = b.toolchains.clang(b);

    const raylib = b.dependency("raylib"); // dependency finds an external dependency, currently
                                           // only over pkg-config

	let dependencies = [raylib];
	let define = [];

	const useFoo = b.option("useFoo", "true");
	const useBar = b.option("useBar", "true");

	

	if (useFoo) {
		console.log("-----------");
		console.log(" USING FOO ");
		console.log("-----------");
		const foo = b.dependency("foo::foo");
		dependencies.push(foo);
		define.push("FOO_ON");
	}

	
	
	if (useBar) {
		console.log("-----------");
		console.log(" USING BAR ");
		console.log("-----------");
		const bar = b.dependency("bar::bar");
		dependencies.push(bar);
		define.push("BAR_ON");
	}
    

    const hello = b.executable("hello", { // Create an executable target
        sources: b.glob("src/**/*.c"),
        dependencies: dependencies,       // Link with Raylib, foo and bar.
                                          // Dependencies works with both external dependencies
                                          // and local targets
        toolchain: clang,
    	includeDirectories: ["src/"],
    	define: define
    });

    return [hello]; // All targets that should be built must be returned
                              // Order doesnt matter, dependency tree will be resolved.
}
