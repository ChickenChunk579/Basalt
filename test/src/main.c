#include <stdio.h>
#include "raylib.h"
#include "hello.h"
#include "foo.h"
#include "bar.h"

int main(void)
{
    const int screenWidth = 800;
    const int screenHeight = 450;

    InitWindow(screenWidth, screenHeight, "raylib [core] example - basic window");

    SetTargetFPS(60);

	printf("HELLO is: %s\n", HELLO);
	foo();
	bar();

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
