package org.polyfrost.plusrenderer;

import static org.junit.Assert.*;

import org.junit.Test;

// Needs the host native: scripts/build-native.sh target/java-resources
public class RendererTest {
    private static final String GLASSES = "{\"format_version\":\"1.12.0\",\"minecraft:geometry\":[{"
            + "\"description\":{\"identifier\":\"geometry.glasses\",\"texture_width\":16,\"texture_height\":16},"
            + "\"bones\":[{\"name\":\"bb_main\",\"pivot\":[0,0,0],"
            + "\"cubes\":[{\"origin\":[-4,27,-5],\"size\":[8,2,1],\"uv\":[0,0]}]}]}]}";

    @Test
    public void aGameHostRoundTripsPartsAndBuildsAMesh() {
        try (Renderer r = new Renderer()) {
            r.loadPlayer(0, false);
            r.setVisible(0, false);
            r.loadAnimation(0, "{\"animations\":{\"nod\":{\"bones\":{\"head\":{\"rotation\":[30,0,0]}}}}}");
            r.play(0, 0);
            r.loadCosmetic(1, GLASSES, "glasses", "", 16, 16);
            r.pose(0);

            float[] d = r.partDelta(0, "head");
            assertEquals(Math.toRadians(30), d[3], 1e-6);
            r.setGamePart("head", 0, 0, 0, d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7], d[8]);
            assertNull(r.partDelta(0, "tail"));

            assertEquals("only the glasses: the hidden player is the game's to draw", 6, r.buildMesh(Renderer.JAVA_MODEL_SPACE));
            assertArrayEquals(new int[] {1, 0, 6, 0}, r.meshRuns());
            assertEquals(6 * 4 * 3, r.meshPositions().length);
            assertEquals(6 * 4, r.meshGlow().length);

            assertTrue(r.build(0, 10, 62, 18, 512, 512, 0) > 0);
            assertEquals(r.positions().length / 2, r.colors().length);
        }
    }

    @Test
    public void errorsBecomeExceptions() {
        try (Renderer r = new Renderer()) {
            try {
                r.loadCosmetic(1, GLASSES, "tail", "", 0, 0);
                fail();
            } catch (IllegalArgumentException e) {
                assertTrue(e.getMessage(), e.getMessage().contains("unknown body slot"));
            }

            r.close();
            try {
                r.pose(0);
                fail();
            } catch (IllegalStateException expected) {
            }
        }
    }

    @Test
    public void bundlePartsPickByKind() {
        String[] names = {"wings.png", "wings.geo.json"};
        assertEquals(1, Renderer.bundlePart(names, Renderer.GEOMETRY));
        assertEquals(0, Renderer.bundlePart(names, Renderer.TEXTURE));
        assertEquals(-1, Renderer.bundlePart(names, Renderer.SHEET));
    }

    @Test
    public void aSkinPosedByFrameIsTheMesh() {
        try (Renderer r = new Renderer()) {
            r.loadPlayer(0, false);
            r.setVisible(0, false);
            r.loadCosmetic(1, GLASSES, "glasses", "", 16, 16);
            int head = r.partId("head");
            int moving = r.queryId("is_moving");
            assertEquals(r.partId("head"), head);

            float[] input = new float[12 + 1];
            input[head * 12 + 6] = 0.5f;
            input[head * 12 + 9] = input[head * 12 + 10] = input[head * 12 + 11] = 1;
            input[12 + moving] = 1;

            r.setGamePart("head", 0, 0, 0, 0, 0, 0, 0.5f, 0, 0, 1, 1, 1);
            r.pose(0);
            int quads = r.buildMesh(Renderer.JAVA_MODEL_SPACE);
            float[] mesh = r.meshPositions();

            int layout = r.layout();
            assertEquals(quads, r.buildSkin());
            assertArrayEquals(r.meshRuns(), r.skinRuns());
            float[] skin = r.skinPositions();
            int[] bones = r.skinBones();

            float[] out = new float[64];
            int boneCount = r.frame(0, input, 1, 1, Renderer.JAVA_MODEL_SPACE, out);
            assertEquals(r.skinBoneCount(), boneCount);
            assertEquals(layout, r.layout());
            for (int v = 0; v < bones.length; v++) {
                int m = bones[v] * 12;
                for (int row = 0; row < 3; row++) {
                    float p = out[m + row * 4] * skin[v * 3] + out[m + row * 4 + 1] * skin[v * 3 + 1]
                            + out[m + row * 4 + 2] * skin[v * 3 + 2] + out[m + row * 4 + 3];
                    assertEquals(mesh[v * 3 + row], p, 1e-5);
                }
            }
            assertEquals("one run, on its first frame", 0, out[boneCount * 12], 0);

            try {
                r.frame(0, input, 1, 1, Renderer.JAVA_MODEL_SPACE, new float[1]);
                fail("a short output must throw");
            } catch (IllegalArgumentException expected) {
                assertTrue(expected.getMessage().contains("out holds 1"));
            }
        }
    }
}
