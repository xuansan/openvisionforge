#pragma once

#include <opencv2/opencv.hpp>
#include <opencv2/geometry/2d.hpp>
#include <opencv2/imgproc.hpp>

#include <algorithm>
#include <filesystem>
#include <iostream>
#include <sstream>
#include <string>
#include <vector>

namespace fs = std::filesystem;

static void fail(const std::string& message) {
    std::cerr << message << std::endl;
    std::exit(2);
}

static std::string json_escape(const std::string& value) {
    std::string out;
    for (char ch : value) {
        if (ch == '\\') {
            out += "\\\\";
        } else if (ch == '"') {
            out += "\\\"";
        } else {
            out += ch;
        }
    }
    return out;
}

static std::string pixel_format(const cv::Mat& image) {
    if (image.channels() == 1) {
        return "Gray8";
    }
    if (image.channels() == 3) {
        return "Bgr8";
    }
    if (image.channels() == 4) {
        return "Bgra8";
    }
    return "Unknown";
}

static cv::Mat read_image(const std::string& path, int flags = cv::IMREAD_UNCHANGED) {
    cv::Mat image = cv::imread(path, flags);
    if (image.empty()) {
        fail("failed to read image: " + path);
    }
    return image;
}

static void ensure_parent(const std::string& path) {
    fs::path target(path);
    if (target.has_parent_path()) {
        fs::create_directories(target.parent_path());
    }
}

static void write_image(const std::string& path, const cv::Mat& image) {
    ensure_parent(path);
    if (!cv::imwrite(path, image)) {
        fail("failed to write image: " + path);
    }
}

static std::string metadata_json(const cv::Mat& image, const std::string& path) {
    std::ostringstream out;
    out << "{\"width\":" << image.cols
        << ",\"height\":" << image.rows
        << ",\"channels\":" << image.channels()
        << ",\"pixel_format\":\"" << pixel_format(image)
        << "\",\"source_uri\":\"" << json_escape(path) << "\"}";
    return out.str();
}

static std::vector<std::string> split_tab(const std::string& line) {
    std::vector<std::string> parts;
    std::string item;
    std::stringstream stream(line);
    while (std::getline(stream, item, '\t')) {
        parts.push_back(item);
    }
    return parts;
}
