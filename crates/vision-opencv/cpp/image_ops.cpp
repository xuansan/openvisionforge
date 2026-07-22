#include "bridge_common.hpp"

static std::string op_metadata(const std::vector<std::string>& args) {
    if (args.size() < 2) {
        fail("usage: metadata <input>");
    }
    cv::Mat image = read_image(args[1]);
    return metadata_json(image, args[1]);
}

static std::string op_write(const std::vector<std::string>& args) {
    if (args.size() < 3) {
        fail("usage: write <input> <output>");
    }
    cv::Mat image = read_image(args[1]);
    write_image(args[2], image);
    return metadata_json(image, args[2]);
}

static std::string op_resize(const std::vector<std::string>& args) {
    if (args.size() < 5) {
        fail("usage: resize <input> <output> <width> <height>");
    }
    cv::Mat image = read_image(args[1]);
    cv::Mat resized;
    cv::resize(image, resized, cv::Size(std::stoi(args[3]), std::stoi(args[4])));
    write_image(args[2], resized);
    return metadata_json(resized, args[2]);
}

static std::string op_crop(const std::vector<std::string>& args) {
    if (args.size() < 7) {
        fail("usage: crop <input> <output> <x> <y> <width> <height>");
    }
    cv::Mat image = read_image(args[1]);
    int x = std::stoi(args[3]);
    int y = std::stoi(args[4]);
    int width = std::stoi(args[5]);
    int height = std::stoi(args[6]);
    cv::Rect roi(x, y, width, height);
    cv::Rect bounds(0, 0, image.cols, image.rows);
    cv::Rect clipped = roi & bounds;
    if (clipped.width <= 0 || clipped.height <= 0) {
        fail("crop rectangle is outside image bounds");
    }
    cv::Mat cropped = image(clipped).clone();
    write_image(args[2], cropped);
    return metadata_json(cropped, args[2]);
}
