// Copyright 2018 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef GARNET_DRIVERS_WLAN_TESTING_PHY_DEVICE_H
#define GARNET_DRIVERS_WLAN_TESTING_PHY_DEVICE_H

#include <ddk/device.h>
#include <wlan/dispatcher/dispatcher.h>
#include <zircon/types.h>

#include "garnet/lib/wlan/fidl2/fidl2.fidl.cc.h"

#include <memory>
#include <mutex>
#include <unordered_map>

namespace wlan {
namespace testing {

class IfaceDevice;

class PhyDevice : public wlan::Phy {
   public:
    PhyDevice(zx_device_t* device);
    virtual ~PhyDevice() = default;

    zx_status_t Bind();

    void Unbind();
    void Release();
    zx_status_t Ioctl(uint32_t op, const void* in_buf, size_t in_len, void* out_buf,
                         size_t out_len, size_t* out_actual);

    virtual void Query(QueryCallback callback) override;
    virtual void CreateIface(CreateIfaceRequest req,
                             CreateIfaceCallback callback) override;
    virtual void DestroyIface(DestroyIfaceRequest req,
                              DestroyIfaceCallback callback) override;

   private:
    zx_status_t Connect(const void* buf, size_t len);
    zx_status_t Query(uint8_t* buf, size_t len, size_t* actual);
    zx_status_t CreateIface(const void* in_buf, size_t in_len, void* out_buf,
            size_t out_len, size_t* out_actual);
    zx_status_t DestroyIface(const void* in_buf, size_t in_len);

    zx_device_t* zxdev_;
    zx_device_t* parent_;

    std::mutex lock_;
    bool dead_ = false;
    std::unique_ptr<wlan::dispatcher::Dispatcher<wlan::Phy>> dispatcher_;
    std::unordered_map<uint16_t, IfaceDevice*> ifaces_;
    // Next available Iface id. Must be checked against the map to prevent overwriting an existing
    // IfaceDevice pointer in the map.
    uint16_t next_id_ = 0;
};

}  // namespace testing
}  // namespace wlan

#endif  // GARNET_DRIVERS_WLAN_TESTING_PHY_DEVICE_H
