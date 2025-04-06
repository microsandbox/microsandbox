import org.jetbrains.kotlin.gradle.tasks.KotlinCompile
import com.vanniktech.maven.publish.SonatypeHost

plugins {
    kotlin("jvm") version "1.9.20"
    id("com.vanniktech.maven.publish") version "0.28.0"
}

group = "dev.microsandbox"
version = "0.1.0"

repositories {
    mavenCentral()
}

dependencies {
    testImplementation(kotlin("test"))
}

tasks.test {
    useJUnitPlatform()
}

// Configure JVM toolchain
kotlin {
    jvmToolchain {
        languageVersion.set(JavaLanguageVersion.of(17))
    }
}

// For backward compatibility, also set the JVM target
tasks.withType<KotlinCompile> {
    kotlinOptions.jvmTarget = "17"
}

java {
    // Ensure Java toolchain matches Kotlin
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(17))
    }
    withJavadocJar()
    withSourcesJar()
}

mavenPublishing {
    coordinates(
        groupId = group as String,
        artifactId = "microsandbox-kotlin",
        version = version as String
    )

    pom {
        name.set("Microsandbox Kotlin SDK")
        description.set("A minimal Kotlin SDK for the Microsandbox project")
        url.set("https://github.com/microsandbox/microsandbox")

        licenses {
            license {
                name.set("Apache License, Version 2.0")
                url.set("http://www.apache.org/licenses/LICENSE-2.0.txt")
                distribution.set("repo")
            }
        }

        developers {
            developer {
                name.set("Microsandbox Team")
                email.set("team@microsandbox.dev")
                organization.set("Microsandbox")
                organizationUrl.set("https://microsandbox.dev")
            }
        }

        scm {
            connection.set("scm:git:git://github.com/microsandbox/microsandbox.git")
            developerConnection.set("scm:git:ssh://github.com:microsandbox/microsandbox.git")
            url.set("https://github.com/microsandbox/microsandbox/tree/main")
        }
    }

    // Configure publishing to Maven Central
    publishToMavenCentral(SonatypeHost.CENTRAL_PORTAL)

    // Enable GPG signing for all publications
    signAllPublications()
}

tasks.wrapper {
    gradleVersion = "8.5"
    distributionType = Wrapper.DistributionType.ALL
}

// Add this task to create a bundle for manual upload
tasks.register<Zip>("createBundle") {
    dependsOn("publishToMavenLocal")

    from(file("${System.getProperty("user.home")}/.m2/repository/dev/microsandbox/microsandbox-kotlin/${version}"))
    include("*.jar", "*.pom", "*.asc")

    archiveFileName.set("bundle-${version}.zip")
    destinationDirectory.set(file("${buildDir}/bundle"))

    doLast {
        println("Bundle created at: ${buildDir}/bundle/bundle-${version}.zip")
        println("You can now upload this bundle to Maven Central via the web interface at https://central.sonatype.com/")
    }
}
