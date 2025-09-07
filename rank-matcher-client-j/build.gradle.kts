plugins {
    id("ecbuild.java-conventions")
}

dependencies {
    compileOnly(libs.netty.http)
    compileOnly(libs.log4j.slf4j2)
    compileOnly(project(":ECCommons"))
}

description = "rank-matcher-client-j"
