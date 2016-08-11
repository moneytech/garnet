// Copyright 2016 The Fuchsia Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#include "msd_intel_buffer.h"
#include "msd.h"

MsdIntelBuffer::MsdIntelBuffer(std::unique_ptr<magma::PlatformBuffer> platform_buf)
    : platform_buf_(std::move(platform_buf))
{
    magic_ = kMagic;
}

MsdIntelBuffer* MsdIntelBuffer::Create(msd_platform_buffer* platform_buffer_token)
{
    auto platform_buf = magma::PlatformBuffer::Create(platform_buffer_token);
    if (!platform_buf)
        return DRETP(nullptr,
                     "MsdIntelBuffer::Create: Could not create platform buffer from token");

    return new MsdIntelBuffer(std::move(platform_buf));
}

//////////////////////////////////////////////////////////////////////////////

msd_buffer* msd_buffer_import(msd_platform_buffer* platform_buf)
{
    return MsdIntelBuffer::Create(platform_buf);
}

void msd_buffer_destroy(msd_buffer* buf) { delete MsdIntelBuffer::cast(buf); }