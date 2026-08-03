function buildFoo(b) {
	const clang = b.toolchains.clang(b);

    const raylib = b.dependency("raylib");
	const foo = b.library("foo", {
		sources: b.glob("src/**/*.c"),
		dependencies: [raylib],
		toolchain: clang,
		isStatic: false,
		includeDirectories: ["src/"],
    	define: []
	});

	return foo;
}

function build(b) {
	return [buildFoo(b)];
}

function pkg(b) {
	return {
		foo: buildFoo(b)
	}
}

