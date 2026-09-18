#include "shroudedit/blueprint_codec.h"

#include <array>
#include <atomic>
#include <bit>
#include <chrono>
#include <cstring>
#include <fstream>
#include <limits>
#include <stdexcept>
#include <type_traits>

#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#endif

namespace shroudedit {
    namespace {
        constexpr std::array<char, 8> Magic{'S','E','B','L','U','E','1','\0'};
        static_assert(std::endian::native == std::endian::little, "The v1 blueprint codec requires a little-endian target");
        struct PendingWrite {
            std::filesystem::path directory, file;
            explicit PendingWrite(const std::filesystem::path& destination) {
                static std::atomic<std::uint64_t> serial{};
                for (int attempt=0;attempt<32;++attempt) {
                    auto candidate=destination;
                    candidate+=".writing-"+std::to_string(std::chrono::steady_clock::now().time_since_epoch().count())+"-"+std::to_string(serial++);
                    if (std::filesystem::create_directory(candidate)) {
                        directory=std::move(candidate); file=directory/"blueprint"; return;
                    }
                }
                throw std::runtime_error("cannot reserve temporary blueprint file");
            }
            ~PendingWrite() {
                std::error_code ignored;
                std::filesystem::remove(file,ignored);
                std::filesystem::remove(directory,ignored);
            }
        };

        template <typename T>
        void Write(std::ostream& stream, const T& value) {
            static_assert(std::is_trivially_copyable_v<T>);
            stream.write(reinterpret_cast<const char*>(&value), sizeof(value));
            if (!stream) throw std::runtime_error("failed to write blueprint");
        }

        template <typename T>
        T Read(std::istream& stream) {
            static_assert(std::is_trivially_copyable_v<T>);
            T value{};
            stream.read(reinterpret_cast<char*>(&value), sizeof(value));
            if (!stream) throw std::runtime_error("truncated blueprint");
            return value;
        }

        void WriteString(std::ostream& stream, const std::string& value) {
            if (value.size() > (std::numeric_limits<std::uint32_t>::max)()) throw std::length_error("blueprint string is too large");
            Write(stream, static_cast<std::uint32_t>(value.size()));
            stream.write(value.data(), static_cast<std::streamsize>(value.size()));
            if (!stream) throw std::runtime_error("failed to write blueprint string");
        }

        std::string ReadString(std::istream& stream, const ValidationLimits& limits) {
            const auto size = Read<std::uint32_t>(stream);
            if (size > limits.maximum_string_bytes) throw std::runtime_error("blueprint string exceeds validation limit");
            std::string value(size, '\0');
            stream.read(value.data(), static_cast<std::streamsize>(size));
            if (!stream) throw std::runtime_error("truncated blueprint string");
            return value;
        }

        void WriteVec3(std::ostream& stream, Vec3 value) { Write(stream, value.x); Write(stream, value.y); Write(stream, value.z); }
        Vec3 ReadVec3(std::istream& stream) { return {Read<double>(stream), Read<double>(stream), Read<double>(stream)}; }
        void WriteQuaternion(std::ostream& stream, Quaternion value) { Write(stream, value.x); Write(stream, value.y); Write(stream, value.z); Write(stream, value.w); }
        Quaternion ReadQuaternion(std::istream& stream) { return {Read<double>(stream), Read<double>(stream), Read<double>(stream), Read<double>(stream)}; }
    }

    void save_blueprint(const Blueprint& blueprint, const std::filesystem::path& path, const ValidationLimits& limits, bool replace) {
        const auto errors = validate(blueprint, limits);
        if (!errors.empty()) throw std::invalid_argument(errors.front().path + ": " + errors.front().message);

        const PendingWrite pending(path);
        const auto& temporary = pending.file;
        {
            std::ofstream stream(temporary, std::ios::binary | std::ios::trunc);
            if (!stream) throw std::runtime_error("cannot create blueprint file");
            stream.write(Magic.data(), Magic.size());
            Write(stream, blueprint.schema_major); Write(stream, blueprint.schema_minor);
            WriteString(stream, blueprint.id); WriteString(stream, blueprint.name);
            WriteVec3(stream, blueprint.extent); WriteVec3(stream, blueprint.anchor);
            Write(stream, static_cast<std::uint8_t>(blueprint.up_axis));
            Write(stream, static_cast<std::uint32_t>(blueprint.props.size()));
            for (const auto& prop : blueprint.props) {
                Write(stream, prop.local_id); WriteString(stream, prop.template_id);
                WriteVec3(stream, prop.transform.position); WriteQuaternion(stream, prop.transform.rotation); WriteVec3(stream, prop.transform.scale);
            }
            Write(stream, static_cast<std::uint32_t>(blueprint.channels.size()));
            for (const auto& channel : blueprint.channels) {
                WriteString(stream, channel.id); Write(stream, channel.dimensions.x); Write(stream, channel.dimensions.y); Write(stream, channel.dimensions.z);
                if (blueprint.schema_minor >= 1) { WriteVec3(stream, channel.origin); WriteVec3(stream, channel.cell_size); }
                Write(stream, static_cast<std::uint64_t>(channel.values.size()));
                for (const auto value : channel.values) Write(stream, value);
                for (const auto coverage : channel.coverage) Write(stream, static_cast<std::uint8_t>(coverage));
            }
            stream.flush();
            if (!stream) throw std::runtime_error("failed to finalize blueprint file");
        }
        bool replaced = false;
#ifdef _WIN32
        replaced = MoveFileExW(std::filesystem::path(temporary).c_str(), path.c_str(), (replace ? MOVEFILE_REPLACE_EXISTING : 0) | MOVEFILE_WRITE_THROUGH) != FALSE;
#else
        std::error_code error;
        if (replace) std::filesystem::rename(temporary, path, error);
        else std::filesystem::create_hard_link(temporary, path, error);
        replaced = !error;
        if (replaced && !replace) std::filesystem::remove(temporary, error);
#endif
        if (!replaced) {
            std::filesystem::remove(temporary);
            throw std::runtime_error("failed to atomically replace blueprint file");
        }
    }

    Blueprint load_blueprint(const std::filesystem::path& path, const ValidationLimits& limits) {
        std::ifstream stream(path, std::ios::binary);
        if (!stream) throw std::runtime_error("cannot open blueprint file");
        std::array<char, Magic.size()> magic{};
        stream.read(magic.data(), magic.size());
        if (!stream || magic != Magic) throw std::runtime_error("invalid blueprint magic");

        Blueprint blueprint;
        blueprint.schema_major = Read<std::uint16_t>(stream); blueprint.schema_minor = Read<std::uint16_t>(stream);
        if (blueprint.schema_major != 1 || blueprint.schema_minor > 1) throw std::runtime_error("unsupported blueprint schema");
        blueprint.id = ReadString(stream, limits); blueprint.name = ReadString(stream, limits);
        blueprint.extent = ReadVec3(stream); blueprint.anchor = ReadVec3(stream);
        blueprint.up_axis = static_cast<UpAxis>(Read<std::uint8_t>(stream));

        const auto propCount = Read<std::uint32_t>(stream);
        if (propCount > limits.maximum_props) throw std::runtime_error("prop count exceeds validation limit");
        blueprint.props.reserve(propCount);
        for (std::uint32_t i = 0; i < propCount; ++i) {
            Prop prop;
            prop.local_id = Read<std::uint64_t>(stream); prop.template_id = ReadString(stream, limits);
            prop.transform.position = ReadVec3(stream); prop.transform.rotation = ReadQuaternion(stream); prop.transform.scale = ReadVec3(stream);
            blueprint.props.push_back(std::move(prop));
        }

        const auto channelCount = Read<std::uint32_t>(stream);
        if (channelCount > 1024) throw std::runtime_error("channel count exceeds validation limit");
        blueprint.channels.reserve(channelCount);
        std::uint64_t remainingCells = limits.maximum_cells;
        for (std::uint32_t i = 0; i < channelCount; ++i) {
            GridChannel channel;
            channel.id = ReadString(stream, limits);
            channel.dimensions = {Read<std::int32_t>(stream), Read<std::int32_t>(stream), Read<std::int32_t>(stream)};
            if (blueprint.schema_minor >= 1) { channel.origin = ReadVec3(stream); channel.cell_size = ReadVec3(stream); }
            const auto cells = Read<std::uint64_t>(stream);
            if (cells > remainingCells) throw std::runtime_error("total cell count exceeds validation limit");
            remainingCells -= cells;
            channel.values.reserve(static_cast<std::size_t>(cells));
            channel.coverage.reserve(static_cast<std::size_t>(cells));
            for (std::uint64_t cell = 0; cell < cells; ++cell) channel.values.push_back(Read<std::uint32_t>(stream));
            for (std::uint64_t cell = 0; cell < cells; ++cell) channel.coverage.push_back(static_cast<Coverage>(Read<std::uint8_t>(stream)));
            blueprint.channels.push_back(std::move(channel));
        }

        if (stream.peek() != std::char_traits<char>::eof()) throw std::runtime_error("unexpected trailing blueprint data");
        const auto errors = validate(blueprint, limits);
        if (!errors.empty()) throw std::runtime_error(errors.front().path + ": " + errors.front().message);
        return blueprint;
    }
}
