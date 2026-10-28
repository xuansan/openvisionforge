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

static std::vector<cv::Point> parse_polygon_points(const std::string& text) {
    std::vector<cv::Point> points;
    std::stringstream stream(text);
    std::string item;
    while (std::getline(stream, item, ';')) {
        std::stringstream point_stream(item);
        std::string x_text;
        std::string y_text;
        if (!std::getline(point_stream, x_text, ',') || !std::getline(point_stream, y_text, ',')) {
            fail("polygon points must use x,y;x,y format");
        }
        points.emplace_back(std::stoi(x_text), std::stoi(y_text));
    }
    if (points.size() < 3) {
        fail("polygon ROI requires at least three points");
    }
    return points;
}

static std::string op_roi_mask(const std::vector<std::string>& args) {
    if (args.size() < 5) {
        fail("usage: roi_mask <input> <output> <rectangle|circle|polygon> <shape args>");
    }
    cv::Mat image = read_image(args[1]);
    cv::Mat mask = cv::Mat::zeros(image.size(), CV_8UC1);
    const std::string& shape = args[3];
    if (shape == "rectangle") {
        if (args.size() < 8) {
            fail("usage: roi_mask <input> <output> rectangle <x> <y> <width> <height>");
        }
        cv::rectangle(
            mask,
            cv::Rect(
                std::stoi(args[4]),
                std::stoi(args[5]),
                std::stoi(args[6]),
                std::stoi(args[7])
            ),
            cv::Scalar(255),
            cv::FILLED
        );
    } else if (shape == "circle") {
        if (args.size() < 7) {
            fail("usage: roi_mask <input> <output> circle <cx> <cy> <radius>");
        }
        cv::circle(
            mask,
            cv::Point(std::stoi(args[4]), std::stoi(args[5])),
            std::stoi(args[6]),
            cv::Scalar(255),
            cv::FILLED
        );
    } else if (shape == "polygon") {
        auto points = parse_polygon_points(args[4]);
        std::vector<std::vector<cv::Point>> polygons{points};
        cv::fillPoly(mask, polygons, cv::Scalar(255));
    } else {
        fail("roi_mask shape must be rectangle, circle or polygon");
    }
    cv::Mat masked = cv::Mat::zeros(image.size(), image.type());
    image.copyTo(masked, mask);
    write_image(args[2], masked);
    const std::string mask_path = args[2] + ".mask.png";
    write_image(mask_path, mask);
    std::string metadata = metadata_json(masked, args[2]);
    metadata.pop_back();
    metadata += ",\"mask_uri\":\"" + json_escape(mask_path) + "\"}";
    return metadata;
}
