#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#ifdef _WIN32
#define LMX_SYS_API __cdecl
#ifndef WINAPI
#define WINAPI __stdcall
#endif
#else
#define LMX_SYS_API
#define WINAPI
#endif

/* Doit correspondre à LMX_CALLBACK_FUNC (WINAPI / stdcall sous Windows). */
typedef int (WINAPI *lmx_sys_callback_fn)(uint32_t event_type, uint32_t event_param);

void LMX_SYS_API lmx_sys_init(void);

uint8_t LMX_SYS_API lmx_sys_close_device(uint32_t *ret_error);
uint8_t LMX_SYS_API lmx_sys_get_pnp_device_info(void *plmx_pnp_dev_info, uint32_t *ret_error);
uint8_t LMX_SYS_API lmx_sys_select_pnp_device(
    uint32_t dw_dev_index,
    void *plmx_pnp_dev_info,
    uint32_t *ret_error
);

uint32_t LMX_SYS_API lmx_sys_reg_notify_callback(uint32_t callback_type, lmx_sys_callback_fn appfunc);
uint32_t LMX_SYS_API lmx_sys_delete_callback_info(uint32_t callback_type);

uint8_t LMX_SYS_API lmx_sys_iso_get_param(uint32_t *pul_param, uint32_t *ret_error);
uint8_t LMX_SYS_API lmx_sys_iso_set_param(uint32_t ul_param, uint32_t *ret_error);
uint8_t LMX_SYS_API lmx_sys_iso_get_capability(void *p_iso_capa_info, uint32_t *ret_error);

uint8_t LMX_SYS_API lmx_sys_ss_get_capability(void *p_ss_capa_info, uint32_t *ret_error);
uint8_t LMX_SYS_API lmx_sys_ss_get_param(uint32_t *pul_param, uint32_t *ret_error);
uint8_t LMX_SYS_API lmx_sys_ss_set_param(uint32_t ul_param, uint32_t *ret_error);

uint8_t LMX_SYS_API lmx_sys_aperture_get_capability(void *p_aperture_capa_info, uint32_t *ret_error);
uint8_t LMX_SYS_API lmx_sys_aperture_get_param(uint32_t *pul_param, uint32_t *ret_error);
uint8_t LMX_SYS_API lmx_sys_aperture_set_param(uint32_t ul_param, uint32_t *ret_error);

uint8_t LMX_SYS_API lmx_sys_camera_mode_get_capability(void *p_camera_mode_capa_info, uint32_t *ret_error);
uint8_t LMX_SYS_API lmx_sys_camera_mode_get_mode_pos(uint32_t *pul_param, uint32_t *ret_error);
uint8_t LMX_SYS_API lmx_sys_camera_mode_set_mode_pos(uint32_t ul_param, uint32_t *ret_error);

uint8_t LMX_SYS_API lmx_sys_rec_ctrl_release(void *lp_rec_ctrl, uint32_t *ret_error);

uint8_t LMX_SYS_API lmx_sys_get_object(
    uint32_t object_handle,
    uint8_t *lp_store_buf,
    uint32_t store_buf_size,
    uint32_t *ret_error
);
uint8_t LMX_SYS_API lmx_sys_get_object_format_type(
    uint32_t obj_handle,
    uint32_t *p_format_type,
    uint32_t *ret_error
);
uint8_t LMX_SYS_API lmx_sys_get_object_data_size(
    uint32_t obj_handle,
    uint64_t *p_data_size,
    uint32_t *ret_error
);
uint8_t LMX_SYS_API lmx_sys_get_object_file_name(
    uint32_t obj_handle,
    void *p_file_name,
    uint32_t *ret_error
);
uint8_t LMX_SYS_API lmx_sys_get_partial_object_ex(
    uint32_t object_handle,
    uint8_t *lp_store_buf,
    uint64_t data_offset,
    uint32_t split_transfer_bytes,
    uint32_t *ret_error
);
uint8_t LMX_SYS_API lmx_sys_skip_object_transfer(uint32_t object_handle, uint32_t *ret_error);

uint8_t LMX_SYS_API lmx_sys_setup_files_config_get_target(uint16_t *pun_param, uint32_t *ret_error);
uint8_t LMX_SYS_API lmx_sys_setup_files_config_set_target(uint16_t un_param, uint32_t *ret_error);

uint8_t LMX_SYS_API lmx_sys_open_session(
    uint32_t ul_connect_ver,
    uint32_t *pul_device_connect_ver,
    uint32_t *ret_error
);
uint8_t LMX_SYS_API lmx_sys_close_session(uint32_t *ret_error);

/* Helpers (appellent les API réelles ; évitent le layout C++ côté Rust). */
uint8_t LMX_SYS_API lmx_sys_list_pnp_models(
    char *out_models_utf8,
    uint32_t model_stride,
    uint32_t max_models,
    uint32_t *out_count,
    uint32_t *ret_error
);
uint8_t LMX_SYS_API lmx_sys_select_pnp_index(uint32_t dw_dev_index, uint32_t *ret_error);

uint8_t LMX_SYS_API lmx_sys_ss_fetch_support(
    uint32_t *out_codes,
    uint32_t max_codes,
    uint32_t *out_count,
    uint32_t *out_cur,
    uint32_t *out_range_min,
    uint32_t *out_range_max,
    uint32_t *ret_error
);
uint8_t LMX_SYS_API lmx_sys_iso_fetch_support(
    uint32_t *out_codes,
    uint32_t max_codes,
    uint32_t *out_count,
    uint32_t *out_cur,
    uint32_t *ret_error
);
uint8_t LMX_SYS_API lmx_sys_aperture_fetch_support(
    uint16_t *out_codes,
    uint32_t max_codes,
    uint32_t *out_count,
    uint16_t *out_cur,
    uint32_t *ret_error
);

uint8_t LMX_SYS_API lmx_sys_rec_ctrl_release_oneshot(uint32_t *ret_error);

uint8_t LMX_SYS_API lmx_sys_ctrl_liveview_start(uint32_t *ret_error);
uint8_t LMX_SYS_API lmx_sys_ctrl_liveview_stop(uint32_t *ret_error);
/* Helper : Get_LiveView_data → JPEG uniquement (hist/posture/level ignorés). */
uint8_t LMX_SYS_API lmx_sys_get_liveview_jpeg(
    uint8_t *jpeg_buf,
    uint32_t jpeg_buf_capacity,
    uint32_t *out_jpeg_size,
    uint32_t *ret_error
);
/* Qualité basse recommandée si la liste n'est pas vide (index 0). */
uint8_t LMX_SYS_API lmx_sys_liveview_set_recom_quality_low(uint32_t *ret_error);

#ifdef __cplusplus
}
#endif
