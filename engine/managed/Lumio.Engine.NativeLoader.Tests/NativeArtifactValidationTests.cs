using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Text.Json;
using Lumio.Engine.NativeLoader;
using Xunit;

namespace Lumio.Engine.NativeLoader.Tests;

public sealed class NativeArtifactValidationTests
{
    [Fact]
    public void AConsistentProducerSidecarCannotOverrideTheCompiledConsumerAbi()
    {
        using var fixture = new ArtifactFixture();
        fixture.Sidecar(new string('0', 64), fixture.Hash);
        var error = Assert.Throws<NativeEngineLoadException>(() => NativeEngineLoader.LoadFromBuildInfo(fixture.Path));
        Assert.Equal(NativeEngineLoadFailure.AbiMismatch, error.Failure);
        Assert.Contains("compiled consumer", error.Message);
    }

    [Fact]
    public void ModifiedBinaryIsRejectedBeforeAttemptingToLoadIt()
    {
        using var fixture = new ArtifactFixture();
        fixture.Sidecar(AbiConstants.DefinitionSha256, fixture.Hash);
        File.AppendAllText(fixture.Path, "modified after build");
        var error = Assert.Throws<NativeEngineLoadException>(() => NativeEngineLoader.LoadFromBuildInfo(fixture.Path));
        Assert.Equal(NativeEngineLoadFailure.BinaryHashMismatch, error.Failure);
    }

    [Fact]
    public void MatchingHashesReachTheNativeImageValidator()
    {
        using var fixture = new ArtifactFixture();
        fixture.Sidecar(AbiConstants.DefinitionSha256.ToUpperInvariant(), fixture.Hash.ToUpperInvariant());
        var error = Assert.Throws<NativeEngineLoadException>(() => NativeEngineLoader.LoadFromBuildInfo(fixture.Path));
        // The bytes deliberately are NOT a library. A correct hash must not bypass native validation.
        Assert.Equal(NativeEngineLoadFailure.InvalidNativeImage, error.Failure);
    }

    [Fact]
    public void EmptyBinaryHashDoesNotDisableArtifactVerification()
    {
        using var fixture = new ArtifactFixture();
        fixture.Sidecar(AbiConstants.DefinitionSha256, "");
        var error = Assert.Throws<NativeEngineLoadException>(() => NativeEngineLoader.LoadFromBuildInfo(fixture.Path));
        Assert.Equal(NativeEngineLoadFailure.BinaryHashMismatch, error.Failure);
    }

    [Theory]
    [InlineData(0)]
    [InlineData(8)]
    [InlineData(16)]
    public void ShortTableIsRejectedBeforeReadingBeyondItsHeader(int declaredSize)
    {
        var address = Marshal.AllocHGlobal(8);
        try
        {
            Marshal.WriteInt32(address, 0, checked((int)AbiConstants.AbiVersion));
            Marshal.WriteInt32(address, 4, declaredSize);
            var error = Assert.Throws<NativeEngineLoadException>(() => NativeEngineLoader.ReadRootApi(address));
            Assert.Equal(NativeEngineLoadFailure.InvalidNativeImage, error.Failure);
        }
        finally { Marshal.FreeHGlobal(address); }
    }

    [Fact]
    public void NullRootTableHasAStableFailureClassification()
    {
        var error = Assert.Throws<NativeEngineLoadException>(() => NativeEngineLoader.ReadRootApi(0));
        Assert.Equal(NativeEngineLoadFailure.ApiTableNull, error.Failure);
    }

    private sealed class ArtifactFixture : IDisposable
    {
        private readonly string _directory = System.IO.Path.Combine(System.IO.Path.GetTempPath(), "lumio-artifact-" + Guid.NewGuid().ToString("N"));
        public string Path { get; }
        public string Hash => Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(Path))).ToLowerInvariant();
        public ArtifactFixture()
        {
            Directory.CreateDirectory(_directory);
            Path = System.IO.Path.Combine(_directory, "not-a-library.bin");
            File.WriteAllText(Path, "test artifact: invalid as an actual native library");
        }
        public void Sidecar(string abiHash, string binarySha256) => File.WriteAllText(NativeBuildInfo.SidecarPath(Path),
            JsonSerializer.Serialize(new { buildId = new string('a', 32), abiHash, binarySha256 }));
        public void Dispose() => Directory.Delete(_directory, recursive: true);
    }
}
