#include <jni.h>
#include <string>
#include <android/log.h>

#define LOG_TAG "AdBlockJNI"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

// External C functions from Rust FFI
extern "C" {
    void* adblock_engine_create();
    void adblock_engine_destroy(void* engine);
    bool adblock_engine_should_block(void* engine, const char* url);
    bool adblock_engine_load_filter_list(void* engine, const char* filter_list);
    char* adblock_engine_get_stats(void* engine);
    void adblock_free_string(char* s);
}

extern "C" JNIEXPORT jlong JNICALL
Java_com_adblock_AdBlockEngine_nativeCreate(JNIEnv* env, jobject /* this */) {
    void* engine = adblock_engine_create();
    if (engine == nullptr) {
        LOGE("Failed to create AdBlock engine");
        return 0;
    }
    LOGI("Created AdBlock engine: %p", engine);
    return reinterpret_cast<jlong>(engine);
}

extern "C" JNIEXPORT void JNICALL
Java_com_adblock_AdBlockEngine_nativeDestroy(JNIEnv* env, jobject /* this */, jlong handle) {
    if (handle == 0) return;
    
    void* engine = reinterpret_cast<void*>(handle);
    adblock_engine_destroy(engine);
    LOGI("Destroyed AdBlock engine: %p", engine);
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_adblock_AdBlockEngine_nativeShouldBlock(JNIEnv* env, jobject /* this */, jlong handle, jstring url) {
    if (handle == 0 || url == nullptr) return JNI_FALSE;
    
    void* engine = reinterpret_cast<void*>(handle);
    const char* url_str = env->GetStringUTFChars(url, nullptr);
    
    bool should_block = adblock_engine_should_block(engine, url_str);
    
    env->ReleaseStringUTFChars(url, url_str);
    
    return should_block ? JNI_TRUE : JNI_FALSE;
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_adblock_AdBlockEngine_nativeLoadFilterList(JNIEnv* env, jobject /* this */, jlong handle, jstring filterList) {
    if (handle == 0 || filterList == nullptr) return JNI_FALSE;
    
    void* engine = reinterpret_cast<void*>(handle);
    const char* filter_list_str = env->GetStringUTFChars(filterList, nullptr);
    
    bool result = adblock_engine_load_filter_list(engine, filter_list_str);
    
    env->ReleaseStringUTFChars(filterList, filter_list_str);
    
    return result ? JNI_TRUE : JNI_FALSE;
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_adblock_AdBlockEngine_nativeGetStats(JNIEnv* env, jobject /* this */, jlong handle) {
    if (handle == 0) return nullptr;
    
    void* engine = reinterpret_cast<void*>(handle);
    char* stats_json = adblock_engine_get_stats(engine);
    
    if (stats_json == nullptr) {
        return nullptr;
    }
    
    jstring result = env->NewStringUTF(stats_json);
    adblock_free_string(stats_json);
    
    return result;
}