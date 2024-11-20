using net.rs64.TexTransCore;
using net.rs64.TexTransCoreEngineForWgpu;
using net.rs64.TexTransCoreEngineForWgpu.Tests;

Console.WriteLine("Test Run");


using var engineTest = new TestTTCEWgpuEngine();
var ctx = engineTest.GetCtx();

// using var rt = ctx.CreateRenderTexture(64, 256);

// ctx.ColorFill(rt, new(1, 1, 1, 1));

// Span<byte> outByte = stackalloc byte[rt.Width * rt.Hight * 4];
// ctx.DownloadTexture(outByte, TexTransCoreTextureFormat.Byte, rt);

// foreach (var v in outByte)
// {
//     Console.Write(v);
// }
