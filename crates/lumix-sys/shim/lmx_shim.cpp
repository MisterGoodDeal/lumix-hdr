#include "lmx_shim.h"

#include <new>

#include "LMX_func_api.h"

extern "C" {

void LMX_SYS_API lmx_sys_init(void) {
    LMX_func_api_Init();
}

uint8_t LMX_SYS_API lmx_sys_close_device(uint32_t *ret_error) {
    return LMX_func_api_Close_Device(ret_error);
}

uint8_t LMX_SYS_API lmx_sys_get_pnp_device_info(void *plmx_pnp_dev_info, uint32_t *ret_error) {
    return LMX_func_api_Get_PnPDeviceInfo(
        static_cast<PLMX_CONNECT_DEVICE_INFO>(plmx_pnp_dev_info),
        ret_error
    );
}

uint8_t LMX_SYS_API lmx_sys_select_pnp_device(
    uint32_t dw_dev_index,
    void *plmx_pnp_dev_info,
    uint32_t *ret_error
) {
    return LMX_func_api_Select_PnPDevice(
        dw_dev_index,
        static_cast<PLMX_CONNECT_DEVICE_INFO>(plmx_pnp_dev_info),
        ret_error
    );
}

uint32_t LMX_SYS_API lmx_sys_reg_notify_callback(uint32_t callback_type, lmx_sys_callback_fn appfunc) {
    return LMX_func_api_Reg_NotifyCallback(callback_type, reinterpret_cast<LMX_CALLBACK_FUNC>(appfunc));
}

uint32_t LMX_SYS_API lmx_sys_delete_callback_info(uint32_t callback_type) {
    return LMX_func_api_Delete_CallBackInfo(callback_type);
}

uint8_t LMX_SYS_API lmx_sys_iso_get_param(uint32_t *pul_param, uint32_t *ret_error) {
    return LMX_func_api_ISO_Get_Param(pul_param, ret_error);
}

uint8_t LMX_SYS_API lmx_sys_iso_set_param(uint32_t ul_param, uint32_t *ret_error) {
    return LMX_func_api_ISO_Set_Param(ul_param, ret_error);
}

uint8_t LMX_SYS_API lmx_sys_iso_get_capability(void *p_iso_capa_info, uint32_t *ret_error) {
    return LMX_func_api_ISO_Get_Capability(
        static_cast<LMX_STRUCT_ISO_CAPA_INFO *>(p_iso_capa_info),
        ret_error
    );
}

uint8_t LMX_SYS_API lmx_sys_ss_get_capability(void *p_ss_capa_info, uint32_t *ret_error) {
    return LMX_func_api_SS_Get_Capability(
        static_cast<LMX_STRUCT_SS_CAPA_INFO *>(p_ss_capa_info),
        ret_error
    );
}

uint8_t LMX_SYS_API lmx_sys_ss_get_param(uint32_t *pul_param, uint32_t *ret_error) {
    return LMX_func_api_SS_Get_Param(pul_param, ret_error);
}

uint8_t LMX_SYS_API lmx_sys_ss_set_param(uint32_t ul_param, uint32_t *ret_error) {
    return LMX_func_api_SS_Set_Param(ul_param, ret_error);
}

uint8_t LMX_SYS_API lmx_sys_aperture_get_capability(void *p_aperture_capa_info, uint32_t *ret_error) {
    return LMX_func_api_Aperture_Get_Capability(
        static_cast<LMX_STRUCT_APERTURE_CAPA_INFO *>(p_aperture_capa_info),
        ret_error
    );
}

uint8_t LMX_SYS_API lmx_sys_aperture_get_param(uint32_t *pul_param, uint32_t *ret_error) {
    return LMX_func_api_Aperture_Get_Param(pul_param, ret_error);
}

uint8_t LMX_SYS_API lmx_sys_aperture_set_param(uint32_t ul_param, uint32_t *ret_error) {
    return LMX_func_api_Aperture_Set_Param(ul_param, ret_error);
}

uint8_t LMX_SYS_API lmx_sys_camera_mode_get_capability(void *p_camera_mode_capa_info, uint32_t *ret_error) {
    return LMX_func_api_CameraMode_Get_Capability(
        static_cast<LMX_STRUCT_RECINFO_CAMERA_MODE_CAPA_INFO *>(p_camera_mode_capa_info),
        ret_error
    );
}

uint8_t LMX_SYS_API lmx_sys_camera_mode_get_mode_pos(uint32_t *pul_param, uint32_t *ret_error) {
    return LMX_func_api_CameraMode_Get_Mode_Pos(pul_param, ret_error);
}

uint8_t LMX_SYS_API lmx_sys_camera_mode_set_mode_pos(uint32_t ul_param, uint32_t *ret_error) {
    return LMX_func_api_CameraMode_Set_Mode_Pos(ul_param, ret_error);
}

uint8_t LMX_SYS_API lmx_sys_rec_ctrl_release(void *lp_rec_ctrl, uint32_t *ret_error) {
    return LMX_func_api_Rec_Ctrl_Release(
        static_cast<LMX_STRUCT_REC_CTRL *>(lp_rec_ctrl),
        ret_error
    );
}

uint8_t LMX_SYS_API lmx_sys_get_object(
    uint32_t object_handle,
    uint8_t *lp_store_buf,
    uint32_t store_buf_size,
    uint32_t *ret_error
) {
    return LMX_func_api_Get_Object(object_handle, lp_store_buf, store_buf_size, ret_error);
}

uint8_t LMX_SYS_API lmx_sys_get_object_format_type(
    uint32_t obj_handle,
    uint32_t *p_format_type,
    uint32_t *ret_error
) {
    return LMX_func_api_Get_Object_FormatType(obj_handle, p_format_type, ret_error);
}

uint8_t LMX_SYS_API lmx_sys_get_object_data_size(
    uint32_t obj_handle,
    uint64_t *p_data_size,
    uint32_t *ret_error
) {
    return LMX_func_api_Get_Object_DataSize(obj_handle, p_data_size, ret_error);
}

uint8_t LMX_SYS_API lmx_sys_get_object_file_name(
    uint32_t obj_handle,
    void *p_file_name,
    uint32_t *ret_error
) {
    return LMX_func_api_Get_Object_FileName(
        obj_handle,
        static_cast<LMX_STRUCT_PTP_ARRAY_STRING *>(p_file_name),
        ret_error
    );
}

uint8_t LMX_SYS_API lmx_sys_get_partial_object_ex(
    uint32_t object_handle,
    uint8_t *lp_store_buf,
    uint64_t data_offset,
    uint32_t split_transfer_bytes,
    uint32_t *ret_error
) {
    return LMX_func_api_Get_Partial_ObjectEx(
        object_handle,
        lp_store_buf,
        data_offset,
        split_transfer_bytes,
        ret_error
    );
}

uint8_t LMX_SYS_API lmx_sys_skip_object_transfer(uint32_t object_handle, uint32_t *ret_error) {
    return LMX_func_api_Skip_Object_Transfer(object_handle, ret_error);
}

uint8_t LMX_SYS_API lmx_sys_setup_files_config_get_target(uint16_t *pun_param, uint32_t *ret_error) {
    return LMX_func_api_SetupFilesConfig_Get_Target(pun_param, ret_error);
}

uint8_t LMX_SYS_API lmx_sys_setup_files_config_set_target(uint16_t un_param, uint32_t *ret_error) {
    return LMX_func_api_SetupFilesConfig_Set_Target(un_param, ret_error);
}

uint8_t LMX_SYS_API lmx_sys_open_session(
    uint32_t ul_connect_ver,
    uint32_t *pul_device_connect_ver,
    uint32_t *ret_error
) {
    return LMX_func_api_Open_Session(ul_connect_ver, pul_device_connect_ver, ret_error);
}

uint8_t LMX_SYS_API lmx_sys_close_session(uint32_t *ret_error) {
    return LMX_func_api_Close_Session(ret_error);
}

static void wide_to_utf8(const WCHAR *src, UINT32 src_len, char *dst, uint32_t dst_len) {
    if (dst == nullptr || dst_len == 0) {
        return;
    }
    dst[0] = '\0';
    if (src == nullptr || src_len == 0) {
        return;
    }
    int written = WideCharToMultiByte(
        CP_UTF8,
        0,
        src,
        static_cast<int>(src_len),
        dst,
        static_cast<int>(dst_len - 1),
        nullptr,
        nullptr
    );
    if (written < 0) {
        written = 0;
    }
    dst[written] = '\0';
}

uint8_t LMX_SYS_API lmx_sys_list_pnp_models(
    char *out_models_utf8,
    uint32_t model_stride,
    uint32_t max_models,
    uint32_t *out_count,
    uint32_t *ret_error
) {
    if (out_count == nullptr) {
        return LMX_BOOL_FALSE;
    }
    *out_count = 0;
    // ~530 Ko : heap obligatoire (évite overflow de pile).
    auto *info = new (std::nothrow) LMX_CONNECT_DEVICE_INFO{};
    if (info == nullptr) {
        return LMX_BOOL_FALSE;
    }
    uint8_t ok = LMX_func_api_Get_PnPDeviceInfo(info, ret_error);
    if (ok != LMX_BOOL_TRUE) {
        delete info;
        return LMX_BOOL_FALSE;
    }
    uint32_t n = info->find_PnpDevice_Count;
    if (n > max_models) {
        n = max_models;
    }
    for (uint32_t i = 0; i < n; ++i) {
        char *slot = out_models_utf8 + (static_cast<size_t>(i) * model_stride);
        wide_to_utf8(
            info->find_PnpDevice_Info[i].dev_ModelName,
            info->find_PnpDevice_Info[i].dev_ModelName_Length,
            slot,
            model_stride
        );
    }
    *out_count = n;
    delete info;
    return LMX_BOOL_TRUE;
}

uint8_t LMX_SYS_API lmx_sys_select_pnp_index(uint32_t dw_dev_index, uint32_t *ret_error) {
    auto *info = new (std::nothrow) LMX_CONNECT_DEVICE_INFO{};
    if (info == nullptr) {
        return LMX_BOOL_FALSE;
    }
    uint8_t ok = LMX_func_api_Get_PnPDeviceInfo(info, ret_error);
    if (ok != LMX_BOOL_TRUE) {
        delete info;
        return LMX_BOOL_FALSE;
    }
    if (dw_dev_index >= info->find_PnpDevice_Count) {
        delete info;
        return LMX_BOOL_FALSE;
    }
    ok = LMX_func_api_Select_PnPDevice(dw_dev_index, info, ret_error);
    delete info;
    return ok;
}

uint8_t LMX_SYS_API lmx_sys_ss_fetch_support(
    uint32_t *out_codes,
    uint32_t max_codes,
    uint32_t *out_count,
    uint32_t *out_cur,
    uint32_t *out_range_min,
    uint32_t *out_range_max,
    uint32_t *ret_error
) {
    LMX_STRUCT_SS_CAPA_INFO info{};
    uint8_t ok = LMX_func_api_SS_Get_Capability(&info, ret_error);
    if (ok != LMX_BOOL_TRUE) {
        return LMX_BOOL_FALSE;
    }
    uint32_t n = info.Capa_Enum.NumOfVal;
    if (n > max_codes) {
        n = max_codes;
    }
    if (n > LMX_DEF_USER_PTP_ARRAY_MAX) {
        n = LMX_DEF_USER_PTP_ARRAY_MAX;
    }
    if (out_codes != nullptr) {
        for (uint32_t i = 0; i < n; ++i) {
            out_codes[i] = info.Capa_Enum.SupportVal[i];
        }
    }
    if (out_count != nullptr) {
        *out_count = n;
    }
    if (out_cur != nullptr) {
        *out_cur = info.CurVal;
    }
    if (out_range_min != nullptr) {
        *out_range_min = info.CurVal_Range.MinVal;
    }
    if (out_range_max != nullptr) {
        *out_range_max = info.CurVal_Range.MaxVal;
    }
    return LMX_BOOL_TRUE;
}

uint8_t LMX_SYS_API lmx_sys_iso_fetch_support(
    uint32_t *out_codes,
    uint32_t max_codes,
    uint32_t *out_count,
    uint32_t *out_cur,
    uint32_t *ret_error
) {
    LMX_STRUCT_ISO_CAPA_INFO info{};
    uint8_t ok = LMX_func_api_ISO_Get_Capability(&info, ret_error);
    if (ok != LMX_BOOL_TRUE) {
        return LMX_BOOL_FALSE;
    }
    uint32_t n = info.Capa_Enum.NumOfVal;
    if (n > max_codes) {
        n = max_codes;
    }
    if (n > LMX_DEF_USER_PTP_ARRAY_MAX) {
        n = LMX_DEF_USER_PTP_ARRAY_MAX;
    }
    if (out_codes != nullptr) {
        for (uint32_t i = 0; i < n; ++i) {
            out_codes[i] = info.Capa_Enum.SupportVal[i];
        }
    }
    if (out_count != nullptr) {
        *out_count = n;
    }
    if (out_cur != nullptr) {
        *out_cur = info.CurVal;
    }
    return LMX_BOOL_TRUE;
}

uint8_t LMX_SYS_API lmx_sys_aperture_fetch_support(
    uint16_t *out_codes,
    uint32_t max_codes,
    uint32_t *out_count,
    uint16_t *out_cur,
    uint32_t *ret_error
) {
    LMX_STRUCT_APERTURE_CAPA_INFO info{};
    uint8_t ok = LMX_func_api_Aperture_Get_Capability(&info, ret_error);
    if (ok != LMX_BOOL_TRUE) {
        return LMX_BOOL_FALSE;
    }
    uint32_t n = info.Capa_Enum.NumOfVal;
    if (n > max_codes) {
        n = max_codes;
    }
    if (n > LMX_DEF_USER_PTP_ARRAY_MAX) {
        n = LMX_DEF_USER_PTP_ARRAY_MAX;
    }
    if (out_codes != nullptr) {
        for (uint32_t i = 0; i < n; ++i) {
            out_codes[i] = info.Capa_Enum.SupportVal[i];
        }
    }
    if (out_count != nullptr) {
        *out_count = n;
    }
    if (out_cur != nullptr) {
        *out_cur = info.CurVal;
    }
    return LMX_BOOL_TRUE;
}

uint8_t LMX_SYS_API lmx_sys_rec_ctrl_release_oneshot(uint32_t *ret_error) {
    LMX_STRUCT_REC_CTRL ctrl{};
    ctrl.CtrlID = LMX_DEF_LIB_TAG_REC_CTRL_RELEASE_ONESHOT;
    return LMX_func_api_Rec_Ctrl_Release(&ctrl, ret_error);
}

uint8_t LMX_SYS_API lmx_sys_ctrl_liveview_start(uint32_t *ret_error) {
    return LMX_func_api_Ctrl_LiveView_Start(ret_error);
}

uint8_t LMX_SYS_API lmx_sys_ctrl_liveview_stop(uint32_t *ret_error) {
    return LMX_func_api_Ctrl_LiveView_Stop(ret_error);
}

uint8_t LMX_SYS_API lmx_sys_get_liveview_jpeg(
    uint8_t *jpeg_buf,
    uint32_t jpeg_buf_capacity,
    uint32_t *out_jpeg_size,
    uint32_t *ret_error
) {
    if (jpeg_buf == nullptr || out_jpeg_size == nullptr || jpeg_buf_capacity == 0) {
        if (ret_error != nullptr) {
            *ret_error = LMX_DEF_ERR_COM_INVALID_PARAM;
        }
        return LMX_BOOL_FALSE;
    }

    LMX_STRUCT_LIVEVIEW_INFO_HISTGRAM histgram_buf{};
    UINT32 histgram_buf_size = sizeof(histgram_buf);
    LMX_STRUCT_LIVEVIEW_INFO_POSTURE posture_buf{};
    UINT32 posture_buf_size = sizeof(posture_buf);
    LMX_STRUCT_LIVEVIEW_INFO_LEVEL level_buf{};
    UINT32 level_buf_size = sizeof(level_buf);
    // Capacité en entrée (certaines builds SDK traitent pJpegSize comme in/out).
    UINT32 jpeg_size = jpeg_buf_capacity;

    uint8_t ok = LMX_func_api_Get_LiveView_data(
        &histgram_buf,
        &histgram_buf_size,
        &posture_buf,
        &posture_buf_size,
        &level_buf,
        &level_buf_size,
        jpeg_buf,
        &jpeg_size,
        ret_error
    );
    if (jpeg_size > jpeg_buf_capacity) {
        jpeg_size = jpeg_buf_capacity;
    }
    *out_jpeg_size = jpeg_size;
    return ok;
}

uint8_t LMX_SYS_API lmx_sys_liveview_set_recom_quality_low(uint32_t *ret_error) {
    LMX_STRUCT_LIVEVIEW_STR_RECOM_IMG recom{};
    uint8_t ok = LMX_func_api_LiveView_Str_Get_Recom_Img(&recom, ret_error);
    if (ok != LMX_BOOL_TRUE) {
        return LMX_BOOL_FALSE;
    }
    if (recom.RecomImgQualityListCount == 0) {
        return LMX_BOOL_TRUE;
    }
    return LMX_func_api_LiveView_Str_Set_Trans_Img(recom.RecomImgQualityListData[0], ret_error);
}

} // extern "C"
