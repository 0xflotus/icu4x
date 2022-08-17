#ifndef ICU4XCodePointSetData_H
#define ICU4XCodePointSetData_H
#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#ifdef __cplusplus
namespace capi {
#endif

typedef struct ICU4XCodePointSetData ICU4XCodePointSetData;
#ifdef __cplusplus
} // namespace capi
#endif
#include "ICU4XDataProvider.h"
#include "diplomat_result_box_ICU4XCodePointSetData_ICU4XError.h"
#ifdef __cplusplus
namespace capi {
extern "C" {
#endif

diplomat_result_box_ICU4XCodePointSetData_ICU4XError ICU4XCodePointSetData_try_get_ascii_hex_digit(const ICU4XDataProvider* provider);

bool ICU4XCodePointSetData_contains(const ICU4XCodePointSetData* self, char32_t cp);
void ICU4XCodePointSetData_destroy(ICU4XCodePointSetData* self);

#ifdef __cplusplus
} // extern "C"
} // namespace capi
#endif
#endif