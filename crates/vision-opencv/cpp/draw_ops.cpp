#include "bridge_common.hpp"

#include <cmath>
#include <fstream>

static std::vector<std::string> split_line_tab(const std::string& line) {
    std::vector<std::string> parts;
    std::string item;
    std::stringstream stream(line);
    while (std::getline(stream, item, '\t')) {
        parts.push_back(item);
    }
    return parts;
}

static int to_int(const std::string& value) {
    return static_cast<int>(std::round(std::stod(value)));
}

static void put_label(cv::Mat& image, const std::string& label, int x, int y, double font_scale, int thickness, const cv::Scalar& color) {
    if (label.empty()) {
        return;
    }
    int baseline = 0;
    cv::Size size = cv::getTextSize(label, cv::FONT_HERSHEY_SIMPLEX, font_scale, thickness, &baseline);
    int top = std::max(0, y - size.height - baseline - 4);
    cv::rectangle(image, cv::Rect(std::max(0, x), top, size.width + 8, size.height + baseline + 6), cv::Scalar(20, 20, 20), cv::FILLED);
    cv::putText(image, label, cv::Point(x + 4, top + size.height + 2), cv::FONT_HERSHEY_SIMPLEX, font_scale, color, thickness, cv::LINE_AA);
}

static std::string op_draw(const std::vector<std::string>& args) {
    if (args.size() < 6) {
        fail("usage: draw <input> <output> <overlay.tsv> <thickness> <font_scale>");
    }
    cv::Mat image = read_image(args[1]);
    cv::Mat out;
    if (image.channels() == 1) {
        cv::cvtColor(image, out, cv::COLOR_GRAY2BGR);
    } else if (image.channels() == 4) {
        cv::cvtColor(image, out, cv::COLOR_BGRA2BGR);
    } else {
        out = image.clone();
    }

    std::ifstream file(args[3]);
    if (!file.is_open()) {
        fail("failed to read overlay file: " + args[3]);
    }

    int thickness = std::max(std::stoi(args[4]), 1);
    double font_scale = std::max(std::stod(args[5]), 0.1);
    const cv::Scalar green(60, 220, 90);
    const cv::Scalar cyan(255, 210, 40);
    const cv::Scalar amber(0, 190, 255);

    std::string line;
    while (std::getline(file, line)) {
        if (line.empty()) {
            continue;
        }
        auto parts = split_line_tab(line);
        if (parts.empty()) {
            continue;
        }
        if (parts[0] == "box" && parts.size() >= 7) {
            int x = to_int(parts[1]);
            int y = to_int(parts[2]);
            int w = std::max(to_int(parts[3]), 1);
            int h = std::max(to_int(parts[4]), 1);
            cv::rectangle(out, cv::Rect(x, y, w, h), green, thickness, cv::LINE_AA);
            std::string label = parts[5];
            if (!parts[6].empty()) {
                label += " " + parts[6];
            }
            put_label(out, label, x, y, font_scale, thickness, green);
        } else if (parts[0] == "contour" && parts.size() >= 8) {
            int x = to_int(parts[1]);
            int y = to_int(parts[2]);
            int w = std::max(to_int(parts[3]), 1);
            int h = std::max(to_int(parts[4]), 1);
            int cx = to_int(parts[5]);
            int cy = to_int(parts[6]);
            cv::rectangle(out, cv::Rect(x, y, w, h), cyan, thickness, cv::LINE_AA);
            cv::drawMarker(out, cv::Point(cx, cy), cyan, cv::MARKER_CROSS, 16, thickness, cv::LINE_AA);
            put_label(out, parts[7], x, y, font_scale, thickness, cyan);
        } else if (parts[0] == "text" && parts.size() >= 4) {
            int x = to_int(parts[1]);
            int y = to_int(parts[2]);
            cv::putText(out, parts[3], cv::Point(x, y), cv::FONT_HERSHEY_SIMPLEX, font_scale, amber, thickness, cv::LINE_AA);
        }
    }

    write_image(args[2], out);
    return metadata_json(out, args[2]);
}
