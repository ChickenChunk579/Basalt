#include <stdio.h>
#include "raylib.h"
#include "hello.h"

#ifdef FOO_ON
#include "foo.h"
#endif

#ifdef BAR_ON
#include "bar.h"
#endif

int main(int argc, char** argv)
{
    const int screenWidth = 800;
    const int screenHeight = 450;

    InitWindow(screenWidth, screenHeight, "raylib [core] example - basic window");

    SetTargetFPS(60);

	printf("HELLO is: %s\n", HELLO);
#ifdef FOO_ON
	foo();
#endif

#ifdef BAR_ON
	bar();
#endif

    while (!WindowShouldClose())
    {
        BeginDrawing();

            ClearBackground(RAYWHITE);

            DrawText("Congrats! You created your first window!", 190, 200, 20, LIGHTGRAY);

        EndDrawing();
    }
    
    CloseWindow();

    return 0;
}
