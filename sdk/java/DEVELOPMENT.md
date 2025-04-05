## Development

### Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/monocore.git
cd monocore/sdk/java

# Build with Maven
mvn clean install
```

### Running Tests

```bash
mvn test
```

### Publishing to Maven Central

Publishing to Maven Central now uses the Central Portal system with the Central Publishing Maven Plugin:

1. Register for a Central Portal account at [central.sonatype.com](https://central.sonatype.com/).

2. Create and verify a namespace for `dev.microsandbox` by proving ownership of the domain.

   - This is done by adding a DNS TXT record to your domain.
   - Follow the instructions provided during namespace registration.

3. Generate a publishing token from the Central Portal:

   - Go to your account settings
   - Navigate to "Access Tokens"
   - Create a new token with publishing permissions

4. Configure your Maven `settings.xml` with the token:

```xml
<settings>
  <servers>
    <server>
      <id>central</id>
      <username>YOUR_USERNAME</username>
      <password>YOUR_PUBLISHING_TOKEN</password>
    </server>
  </servers>
</settings>
```

5. Sign your artifacts with GPG:

   - Generate a key pair: `gpg --gen-key`
   - Distribute your public key: `gpg --keyserver keyserver.ubuntu.com --send-keys YOUR_KEY_ID`

6. Deploy to Central Repository using the central-publishing-maven-plugin:

```bash
# Deploy with the release profile activated for GPG signing
mvn clean deploy -P release
```

The plugin is configured to automatically publish your artifacts and wait until they are fully published to Maven Central.

You can also use the plugin directly:

```bash
# Run only the publish goal of the plugin
mvn org.sonatype.central:central-publishing-maven-plugin:publish -P release
```

For more detailed instructions, refer to the [Maven Central documentation](https://central.sonatype.org/publish/publish-maven/).
