function build(b) {
	const clang = b.toolchain({
		cc: b.findProgram("clang"),
		cxx: b.findProgram("clang++"),
		ld: b.findProgramOr(["mold", "ld.lld", "ld"]),
		ar: b.findProgram("ar")
	});

    const raylib = b.dependency("raylib");

	const foo = b.library("foo", {
		sources: b.glob("foo/src/**/*.c"),
		dependencies: [raylib],
		toolchain: clang,
		isStatic: true,
		includeDirectories: ["foo/src/"],
    	define: {}
	});

	const bar = b.library("bar", {
		sources: b.glob("bar/src/**/*.c"),
		dependencies: [raylib],
		toolchain: clang,
		isStatic: true,
		includeDirectories: ["bar/src/"],
    	define: {}
	});
		
    const hello = b.executable("hello", {
        sources: b.glob("src/**/*.c"),
        dependencies: [raylib, foo, bar],
        toolchain: clang,
    	includeDirectories: ["src/"],
    	define: {}
    });

    return [hello, foo, bar];
}
