## Development

## Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/monocore.git
cd monocore/sdk/csharp

# Restore dependencies
dotnet restore
```

## Running Tests

```bash
dotnet test
```

## Building the Package

```bash
dotnet build --configuration Release
```

## Publishing to NuGet

1. Create an API key on [NuGet](https://www.nuget.org/)

2. Create a NuGet package:

```bash
dotnet pack --configuration Release
```

3. Publish to NuGet:

```bash
dotnet nuget push src/Microsandbox/bin/Release/Microsandbox.0.1.0.nupkg --api-key YOUR_API_KEY --source https://api.nuget.org/v3/index.json
```

Make sure you have registered for an account on [NuGet.org](https://www.nuget.org/) and have created an API key.
