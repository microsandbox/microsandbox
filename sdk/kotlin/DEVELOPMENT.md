## Development

### Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/monocore.git
cd monocore/sdk/kotlin
```

### Gradle Wrapper Setup

If the Gradle wrapper isn't already included in the repository, you'll need to set it up:

```bash
# Install Gradle locally if you don't have it
# On macOS with Homebrew:
brew install gradle

# On Ubuntu/Debian:
sudo apt install gradle

# Generate the Gradle wrapper
gradle wrapper --gradle-version 8.5

# Now you can use the wrapper for all subsequent commands
./gradlew build
```

The Gradle wrapper (`gradlew` or `gradlew.bat` for Windows) allows anyone to run the build without having to install Gradle first. It ensures everyone uses the same Gradle version.

### Running Tests

```bash
./gradlew test
```

### Publishing to Maven Central

Publishing to Maven Central uses the Vanniktech Maven Publish Plugin which supports the new Central Portal system:

1. Register for a Central Portal account at [central.sonatype.com](https://central.sonatype.com/).

2. Create and verify a namespace for `dev.microsandbox` by proving ownership of the domain.

   - This is done by adding a DNS TXT record to your domain.
   - Follow the instructions provided during namespace registration.

3. Generate a publishing token from the Central Portal:

   - Go to your account settings
   - Navigate to "Access Tokens"
   - Create a new token with publishing permissions

4. Configure your Gradle properties with the token and signing information:

Create a `gradle.properties` file in your project root (or add to your global `~/.gradle/gradle.properties`):

```properties
# Maven Central credentials
mavenCentralUsername=your-central-portal-username
mavenCentralPassword=your-central-portal-token

# GPG Signing
signing.keyId=LAST_8_CHARS_OF_KEY_ID
signing.password=YOUR_KEY_PASSPHRASE
signing.key=-----BEGIN PGP PRIVATE KEY BLOCK-----\n\
\n\
lQdGBGRJTqMBEADHhwTW0m4JCn+cG7Oi7HvyK3Mj+HhZm9TpGGh2FZXXuMvvMXHe\n\
... (continue with each line ending with \n\) ...\n\
=XXXX\n\
-----END PGP PRIVATE KEY BLOCK-----
```

Note: Each line must end with `\n\` and have no spaces after the backslashes.

5. Set up GPG signing:

   - Generate a key pair: `gpg --gen-key`
   - Find your key ID: `gpg --list-keys --keyid-format SHORT`
   - Export your secret key ring: `gpg --export-secret-keys --output secring.gpg`
   - Distribute your public key: `gpg --keyserver keyserver.ubuntu.com --send-keys YOUR_KEY_ID`

6. Deploy to Maven Central:

```bash
# This will publish to Maven Central and automatically release
./gradlew publishAndReleaseToMavenCentral
```

For more detailed instructions, refer to:

- [Vanniktech Maven Publish Plugin documentation](https://vanniktech.github.io/gradle-maven-publish-plugin/)
- [Maven Central documentation](https://central.sonatype.org/publish/publish-guide/)
