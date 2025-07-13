//
//  AdBlock-Bridging-Header.h
//  AdBlock
//
//  Bridging header for Rust FFI functions
//

#ifndef AdBlock_Bridging_Header_h
#define AdBlock_Bridging_Header_h

#import <Foundation/Foundation.h>

// FFI function declarations
void* adblock_engine_create(void);
void adblock_engine_destroy(void* engine);
bool adblock_engine_should_block(void* engine, const char* url);
bool adblock_engine_load_filter_list(void* engine, const char* filter_list);
char* adblock_engine_get_stats(void* engine);
bool adblock_engine_reset_stats(void* engine);
void adblock_free_string(char* s);

#endif /* AdBlock_Bridging_Header_h */