plugins {
    id("ecbuild.java-conventions")
}

dependencies {
    compileOnly(libs.netty.all)
    compileOnly(libs.log4j.api)
    compileOnly(libs.log4j.slf4j2)
    compileOnly(project(":ECCommons"))
}

description = "rank-matcher-client-j"
