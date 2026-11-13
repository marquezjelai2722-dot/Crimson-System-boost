package com.crimson.boost;

interface IAdvancedService {
    String setGameMode(String packageName, String mode);
    String trimSystemCaches();
    String compileSelectedApp(String packageName);
}
