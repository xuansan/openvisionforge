#include "bridge_common.hpp"

static std::string op_rotate(const std::vector<std::string>& args) {
    if (args.size() < 4) {
        fail("usage: rotate <input> <output> <right|left|180>");
    }
    cv::Mat image = read_image(args[1]);
    cv::Mat rotated;
    const std::string& direction = args[3];
    if (direction == "right" || direction == "clockwise" || direction == "90cw") {
        cv::rotate(image, rotated, cv::ROTATE_90_CLOCKWISE);
    } else if (direction == "left" || direction == "counterclockwise" || direction == "90ccw") {
        cv::rotate(image, rotated, cv::ROTATE_90_COUNTERCLOCKWISE);
    } else if (direction == "180") {
        cv::rotate(image, rotated, cv::ROTATE_180);
    } else {
        fail("unsupported rotate direction: " + direction);
    }
    write_image(args[2], rotated);
    return metadata_json(rotated, args[2]);
}

static std::string op_flip(const std::vector<std::string>& args) {
    if (args.size() < 4) {
        fail("usage: flip <input> <output> <horizontal|vertical|both>");
    }
    cv::Mat image = read_image(args[1]);
    cv::Mat flipped;
    int code;
    const std::string& mode = args[3];
    if (mode == "horizontal" || mode == "x") {
        code = 1;
    } else if (mode == "vertical" || mode == "y") {
        code = 0;
    } else if (mode == "both" || mode == "xy") {
        code = -1;
    } else {
        fail("unsupported flip mode: " + mode);
    }
    cv::flip(image, flipped, code);
    write_image(args[2], flipped);
    return metadata_json(flipped, args[2]);
}
